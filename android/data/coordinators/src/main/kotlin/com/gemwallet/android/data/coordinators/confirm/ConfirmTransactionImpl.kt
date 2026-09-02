package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.cases.ConfirmTransaction
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.domains.confirm.asset
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CoroutineScope
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemExecuteResult
import uniffi.gemstone.GemSendInput
import uniffi.gemstone.GemSignerError
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class ConfirmTransactionImpl(
    private val confirmService: GemConfirmTransferService,
    private val createTransactionsCase: CreateTransaction,
) : ConfirmTransaction {
    override suspend fun invoke(
        signerParams: SignerParams,
        session: Session,
        assetInfo: AssetInfo,
        scope: CoroutineScope,
        simulation: SimulationResult?,
    ): String {
        val result = try {
            withContext(Dispatchers.IO) { confirmService.execute(signerParams.toSendInput(session.wallet, simulation)) }
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

}

internal fun GemSignerError.toConfirmError(chain: Chain): ConfirmError = when (this) {
    GemSignerError.DustThreshold -> ConfirmError.DustThreshold(chain)
    is GemSignerError.InvalidInput,
    is GemSignerError.SigningError,
    GemSignerError.InsufficientFunds,
    GemSignerError.SwapValueBelowMinimum -> ConfirmError.SignFail
}
