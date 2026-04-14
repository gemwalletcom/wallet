package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.blockchain.operators.LoadPrivateKeyOperator
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.SignClientProxy
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.util.Arrays

class ConfirmTransactionImpl(
    private val passwordStore: PasswordStore,
    private val loadPrivateKeyOperator: LoadPrivateKeyOperator,
    private val signClient: SignClientProxy,
    private val broadcastService: BroadcastService,
    private val createTransactionsCase: CreateTransaction,
    private val assetsRepository: AssetsRepository,
) : ConfirmTransaction {

    override suspend fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
    ): ConfirmTransaction.Result {
        val account = assetInfo.owner ?: throw ConfirmError.TransactionIncorrect

        val signs = sign(signerParams, session, assetInfo)
        if (signs.isEmpty()) {
            throw IllegalStateException("Not implemented")
        }

        if (signerParams.input is ConfirmParams.TransferParams.Generic) {
            if (!(signerParams.input as ConfirmParams.TransferParams.Generic).isSendable) {
                val hash = String(signs.firstOrNull() ?: byteArrayOf())
                return ConfirmTransaction.Result(txHash = hash, finishRoute = "")
            }
        }

        var lastHash = ""
        for (sign in signs) {
            val txHash = broadcastService.send(account, sign, signerParams.input.getTxType())
            if (!sign.contentEquals(signs.last())) {
                delay(500)
            } else {
                lastHash = txHash
                addTransaction(txHash, signerParams, assetInfo, session)
                scope.launch(Dispatchers.IO) { addRecent(assetInfo, signerParams.input) }
            }
        }

        val finishRoute = getFinishRoute(signerParams.input)
        return ConfirmTransaction.Result(txHash = lastHash, finishRoute = finishRoute)
    }

    private suspend fun sign(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
    ): List<ByteArray> {
        val key = loadPrivateKeyOperator(
            session.wallet,
            assetInfo.id().chain,
            passwordStore.getPassword(session.wallet.id)
        )
        val sign = try {
            signClient.signTransaction(
                params = signerParams,
                privateKey = key
            )
        } catch (_: Throwable) {
            throw ConfirmError.SignFail
        } finally {
            Arrays.fill(key, 0)
        }
        return sign
    }

    private suspend fun addTransaction(
        txHash: String,
        signerParams: SignerParams,
        assetInfo: AssetInfo,
        session: Session,
    ) {
        val destinationAddress = signerParams.input.destination()?.address ?: ""

        createTransactionsCase.createTransaction(
            hash = txHash,
            walletId = session.wallet.id,
            assetId = assetInfo.id(),
            owner = assetInfo.owner!!,
            to = destinationAddress,
            state = TransactionState.Pending,
            fee = signerParams.fee(),
            amount = signerParams.finalAmount,
            memo = signerParams.input.memo() ?: "",
            type = signerParams.input.getTxType(),
            metadata = assembleMetadata(signerParams),
            direction = if (destinationAddress == assetInfo.owner!!.address) {
                TransactionDirection.SelfTransfer
            } else {
                TransactionDirection.Outgoing
            },
            blockNumber = signerParams.data().chainData.blockNumber()
        )
    }

    private suspend fun addRecent(assetInfo: AssetInfo, request: ConfirmParams) {
        val walletId = assetInfo.walletId ?: return
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

    private fun assembleMetadata(signerParams: SignerParams) = when (val input = signerParams.input) {
        is ConfirmParams.SwapParams -> {
            jsonEncoder.encodeToString(
                TransactionSwapMetadata(
                    fromAsset = input.fromAsset.id,
                    toAsset = input.toAsset.id,
                    fromValue = input.fromAmount.toString(),
                    toValue = input.toAmount.toString(),
                    provider = input.protocolId,
                )
            )
        }
        is ConfirmParams.NftParams -> jsonEncoder.encodeToString(
            TransactionNFTTransferMetadata(input.nftAsset.id, input.nftAsset.name)
        )
        else -> null
    }

    private fun getFinishRoute(input: ConfirmParams): String = when (input) {
        is ConfirmParams.Stake -> "stake"
        is ConfirmParams.SwapParams,
        is ConfirmParams.TokenApprovalParams -> "swap"
        is ConfirmParams.TransferParams,
        is ConfirmParams.Activate,
        is ConfirmParams.NftParams,
        is ConfirmParams.PerpetualParams -> "asset"
    }
}
