package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import uniffi.gemstone.transactionMetadataBlockNumber
import com.wallet.core.primitives.TransactionResourceTypeMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.swap.ApprovalData
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemSignerError
import uniffi.gemstone.GemSignedTransaction
import uniffi.gemstone.GemstoneException
import java.math.BigInteger

class ConfirmTransactionImpl(
    private val passwordStore: PasswordStore,
    private val signTransactionOperator: GemSignTransactionOperator,
    private val confirmService: GemConfirmServiceInterface,
    private val createTransactionsCase: CreateTransaction,
    private val recentAssetsService: RecentAssetsService,
) : ConfirmTransaction {
    override suspend fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
        transactionAssetId: AssetId?,
    ): String {
        val account = assetInfo.owner ?: throw ConfirmError.TransactionIncorrect

        val signedTransactions = sign(signerParams, session)
        if (signedTransactions.isEmpty()) throw ConfirmError.SignFail

        if ((signerParams.input as? ConfirmParams.TransferParams.Generic)?.isSendable == false) {
            return signedTransactions.first().data
        }

        signedTransactions.forEach { signedTransaction ->
            val approval = signerParams.input.approvalData(signedTransaction.transactionType.toPrimitives())
            if (approval != null && approval.value.toBigIntegerOrNull() == null) {
                throw ConfirmError.TransactionIncorrect
            }
        }

        val hashes = try {
            confirmService.broadcast(signerParams.input.toDto(), signedTransactions)
        } catch (error: GemConfirmException.Broadcast) {
            addTransactions(error.hashes, signedTransactions, signerParams, session, assetInfo, account, transactionAssetId)
            throw error
        }
        addTransactions(hashes, signedTransactions, signerParams, session, assetInfo, account, transactionAssetId)
        scope.launch(Dispatchers.IO) { addRecent(assetInfo, signerParams.input) }

        return hashes.last()
    }

    private suspend fun addTransactions(
        hashes: List<String>,
        signedTransactions: List<GemSignedTransaction>,
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        account: Account,
        transactionAssetId: AssetId?,
    ) {
        for ((index, transactionHash) in hashes.withIndex()) {
            val isFinalTransaction = index == signedTransactions.lastIndex
            val transactionType = signedTransactions[index].transactionType.toPrimitives()
            val approval = signerParams.input.approvalData(transactionType)
            val approvalAmount = approval?.let {
                it.value.toBigIntegerOrNull() ?: throw ConfirmError.TransactionIncorrect
            }
            addTransaction(
                transactionHash = transactionHash,
                signerParams = signerParams,
                assetInfo = assetInfo,
                account = account,
                session = session,
                transactionAssetId = transactionAssetId,
                transactionType = transactionType,
                approval = approval,
                approvalAmount = approvalAmount,
                isFinalTransaction = isFinalTransaction,
            )
        }
    }

    private suspend fun sign(
        signerParams: SignerParams,
        session: Session,
    ): List<GemSignedTransaction> {
        return try {
            signTransactionOperator(
                session.wallet,
                signerParams,
                passwordStore.getPassword(session.wallet.id.id),
            )
        } catch (error: GemstoneException.SignerException) {
            throw error.error.toConfirmError(signerParams.input.assetId.chain)
        } catch (_: Throwable) {
            throw ConfirmError.SignFail
        }
    }

    private suspend fun addTransaction(
        transactionHash: String,
        signerParams: SignerParams,
        assetInfo: AssetInfo,
        account: Account,
        session: Session,
        transactionAssetId: AssetId?,
        transactionType: TransactionType,
        approval: ApprovalData?,
        approvalAmount: BigInteger?,
        isFinalTransaction: Boolean,
    ) {
        val assetId: AssetId
        val destinationAddress: String
        val amount: BigInteger
        val memo: String?
        val metadata: String?
        if (approval != null) {
            assetId = transactionAssetId ?: AssetId(signerParams.input.asset.id.chain, approval.token)
            destinationAddress = approval.spender
            amount = requireNotNull(approvalAmount)
            memo = null
            metadata = null
        } else {
            if (!isFinalTransaction) return
            assetId = transactionAssetId ?: assetInfo.id()
            destinationAddress = signerParams.input.destination()?.address.orEmpty()
            amount = signerParams.finalAmount
            memo = signerParams.input.memo().orEmpty()
            metadata = signerParams.input.toTransactionMetadataJson()
        }

        createTransactionsCase.createTransaction(
            hash = transactionHash,
            walletId = session.wallet.id,
            assetId = assetId,
            owner = account,
            to = destinationAddress,
            state = TransactionState.Pending,
            fee = signerParams.fee(),
            amount = amount,
            memo = memo,
            type = transactionType,
            metadata = metadata,
            direction = if (destinationAddress == account.address) {
                TransactionDirection.SelfTransfer
            } else {
                TransactionDirection.Outgoing
            },
            blockNumber = transactionMetadataBlockNumber(signerParams.data().metadata)
        )
    }

    private suspend fun addRecent(assetInfo: AssetInfo, request: ConfirmParams) {
        val walletId = assetInfo.walletId?.id ?: return
        val type = when (request) {
            is ConfirmParams.SwapParams -> RecentType.Swap
            is ConfirmParams.TransferParams -> RecentType.Send
            else -> return
        }
        val toAssetId = if (request is ConfirmParams.SwapParams) {
            request.toAsset.id
        } else {
            null
        }
        try {
            recentAssetsService.addRecentActivity(assetInfo.id(), walletId, type, toAssetId)
        } catch (_: Throwable) {}
    }
}

internal fun GemSignerError.toConfirmError(chain: Chain): ConfirmError = when (this) {
    GemSignerError.DustThreshold -> ConfirmError.DustThreshold(chain)
    is GemSignerError.InvalidInput,
    is GemSignerError.SigningError,
    GemSignerError.InsufficientFunds,
    GemSignerError.SwapValueBelowMinimum -> ConfirmError.SignFail
}

internal fun ConfirmParams.toTransactionMetadataJson(): String? = when (this) {
    is ConfirmParams.SwapParams -> {
        jsonEncoder.encodeToString(
            TransactionSwapMetadata(
                fromAsset = fromAsset.id,
                toAsset = toAsset.id,
                fromValue = fromAmount.toString(),
                toValue = toAmount.toString(),
                provider = protocolId,
            )
        )
    }
    is ConfirmParams.NftParams -> jsonEncoder.encodeToString(
        TransactionNFTTransferMetadata(nftAsset.id, nftAsset.name)
    )
    is ConfirmParams.Stake.Freeze -> jsonEncoder.encodeToString(
        TransactionResourceTypeMetadata(resource)
    )
    is ConfirmParams.Stake.Unfreeze -> jsonEncoder.encodeToString(
        TransactionResourceTypeMetadata(resource)
    )
    else -> null
}
