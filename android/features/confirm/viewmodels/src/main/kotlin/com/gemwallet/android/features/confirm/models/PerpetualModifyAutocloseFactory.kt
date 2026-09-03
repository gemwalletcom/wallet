package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualModifyConfirmData
import uniffi.gemstone.GemAutocloseSummary
import uniffi.gemstone.GemConfirmTransferServiceInterface

object PerpetualModifyAutocloseFactory {

    private const val ClearedPlaceholder: String = "-"

    fun create(
        data: PerpetualModifyConfirmData,
        confirmService: GemConfirmTransferServiceInterface,
    ): ConfirmDetailElement.PerpetualModifyAutoclose? =
        confirmService.autocloseSummary(data.toJson())?.let(::element)

    internal fun element(summary: GemAutocloseSummary): ConfirmDetailElement.PerpetualModifyAutoclose {
        val formatter = CurrencyFormatter(currency = Currency.USD)
        return ConfirmDetailElement.PerpetualModifyAutoclose(
            takeProfitText = summary.takeProfit?.let(formatter::string)
                ?: ClearedPlaceholder.takeIf { summary.takeProfitCleared },
            stopLossText = summary.stopLoss?.let(formatter::string)
                ?: ClearedPlaceholder.takeIf { summary.stopLossCleared },
        )
    }
}
