package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.cases.ConfirmTransaction
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.data.adapters.assets.RecentAssetsService
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.toTransferData
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemExecuteResult
import uniffi.gemstone.GemSendInput
import uniffi.gemstone.GemSignerError
import uniffi.gemstone.GemTransactionSigner

class ConfirmTransactionImpl(
    private val signer: GemTransactionSigner,
    private val confirmService: GemConfirmServiceInterface,
    private val createTransactionsCase: CreateTransaction,
    private val recentAssetsService: RecentAssetsService,
) : ConfirmTransaction {
    override suspend fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
        simulation: SimulationResult?,
    ): String {
        val result = try {
            confirmService.execute(signerParams.toSendInput(session.wallet, simulation), signer)
        } catch (error: GemConfirmException.Broadcast) {
            createTransactionsCase.trackPendingTransactions()
            throw error
        } catch (error: GemConfirmException.Sign) {
            throw error.error.toConfirmError(signerParams.input.assetId.chain)
        }
        return when (result) {
            is GemExecuteResult.Signed -> result.data.first()
            is GemExecuteResult.Sent -> {
                createTransactionsCase.trackTransactions(session.wallet.id, result.transactions.map { it.decodeJson<Transaction>() })
                scope.launch(Dispatchers.IO) { addRecent(assetInfo, signerParams.input) }
                result.hashes.last()
            }
        }
    }

    private fun SignerParams.toSendInput(wallet: Wallet, simulation: SimulationResult?): GemSendInput = GemSendInput(
        wallet = wallet.toJson(),
        transfer = input.toTransferData(),
        value = finalAmount.toString(),
        fee = confirmData.fee,
        networkFee = fee.amount.toString(),
        metadata = confirmData.metadata,
        simulation = simulation?.toJson(),
    )

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
