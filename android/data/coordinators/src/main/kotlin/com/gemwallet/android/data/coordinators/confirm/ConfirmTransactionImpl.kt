package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.model.blockNumber
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.TransactionResourceTypeMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.TransactionSwapMetadata
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class ConfirmTransactionImpl(
    private val passwordStore: PasswordStore,
    private val signTransactionOperator: GemSignTransactionOperator,
    private val broadcastService: BroadcastService,
    private val createTransactionsCase: CreateTransaction,
    private val assetsRepository: AssetsRepository,
) : ConfirmTransaction {

    override suspend fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
    ): String {
        val account = assetInfo.owner ?: throw ConfirmError.TransactionIncorrect

        val signs = sign(signerParams, session)
        if (signs.isEmpty()) {
            throw IllegalStateException("Not implemented")
        }

        if (signerParams.input is ConfirmParams.TransferParams.Generic) {
            if (!(signerParams.input as ConfirmParams.TransferParams.Generic).isSendable) {
                return String(signs.firstOrNull() ?: byteArrayOf())
            }
        }

        var lastHash = ""
        for ((index, sign) in signs.withIndex()) {
            val transactionHash = broadcastService.send(account, sign, signerParams.input.getTransactionType())
            if (index < signs.lastIndex) {
                delay(500)
            } else {
                lastHash = transactionHash
                addTransaction(transactionHash, signerParams, assetInfo, account, session)
                scope.launch(Dispatchers.IO) { addRecent(assetInfo, signerParams.input) }
            }
        }

        return lastHash
    }

    private suspend fun sign(
        signerParams: SignerParams,
        session: Session,
    ): List<ByteArray> {
        return try {
            signTransactionOperator(
                session.wallet,
                signerParams,
                passwordStore.getPassword(session.wallet.id.id),
            )
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
    ) {
        val destinationAddress = signerParams.input.destination()?.address ?: ""

        createTransactionsCase.createTransaction(
            hash = transactionHash,
            walletId = session.wallet.id,
            assetId = assetInfo.id(),
            owner = account,
            to = destinationAddress,
            state = TransactionState.Pending,
            fee = signerParams.fee(),
            amount = signerParams.finalAmount,
            memo = signerParams.input.memo() ?: "",
            type = signerParams.input.getTransactionType(),
            metadata = assembleMetadata(signerParams),
            direction = if (destinationAddress == account.address) {
                TransactionDirection.SelfTransfer
            } else {
                TransactionDirection.Outgoing
            },
            blockNumber = signerParams.data().metadata.blockNumber()
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
            assetsRepository.addRecentActivity(assetInfo.id(), walletId, type, toAssetId)
        } catch (_: Throwable) {}
    }

    private fun assembleMetadata(signerParams: SignerParams) =
        signerParams.input.toTransactionMetadataJson()

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
