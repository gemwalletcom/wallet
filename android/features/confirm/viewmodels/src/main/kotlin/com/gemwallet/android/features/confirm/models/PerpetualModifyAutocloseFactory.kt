package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.theme.Placeholder
import com.wallet.core.primitives.Currency
import uniffi.gemstone.PerpetualModifyConfirmData
import uniffi.gemstone.GemAutocloseSummary
import uniffi.gemstone.GemConfirmSessionInterface

object PerpetualModifyAutocloseFactory {

    fun create(
        data: PerpetualModifyConfirmData,
        session: GemConfirmSessionInterface,
    ): ConfirmDetailElement.PerpetualModifyAutoclose? =
        session.autocloseSummary(data)?.let(::element)

    internal fun element(summary: GemAutocloseSummary): ConfirmDetailElement.PerpetualModifyAutoclose {
        val formatter = CurrencyFormatter(currency = Currency.USD)
        return ConfirmDetailElement.PerpetualModifyAutoclose(
            takeProfitText = summary.takeProfit?.let(formatter::string)
                ?: Placeholder.empty.takeIf { summary.takeProfitCleared },
            stopLossText = summary.stopLoss?.let(formatter::string)
                ?: Placeholder.empty.takeIf { summary.stopLossCleared },
        )
    }
}
