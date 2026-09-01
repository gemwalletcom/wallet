package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.cases.ConfirmTransaction
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.domains.confirm.toAsset
import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransactionInputType
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
            throw error.error.toConfirmError(signerParams.input.transfer.inputType.asset.id.chain)
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
        confirm = confirmData,
        value = finalAmount.toString(),
        networkFee = fee.amount.toString(),
        simulation = simulation?.toJson(),
    )

    private suspend fun addRecent(assetInfo: AssetInfo, input: GemConfirmInput) {
        val walletId = assetInfo.walletId?.id ?: return
        val inputType = input.transfer.inputType
        val type = when (inputType) {
            is GemTransactionInputType.Swap -> RecentType.Swap
            is GemTransactionInputType.Transfer,
            is GemTransactionInputType.Deposit,
            is GemTransactionInputType.Withdrawal,
            is GemTransactionInputType.Generic -> RecentType.Send
            else -> return
        }
        val toAssetId = inputType.toAsset?.id
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
