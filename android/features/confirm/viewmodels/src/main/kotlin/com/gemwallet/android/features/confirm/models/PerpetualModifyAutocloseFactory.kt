package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.model.text
import com.gemwallet.android.ui.theme.Placeholder
import uniffi.gemstone.PerpetualModifyConfirmData
import uniffi.gemstone.GemAutocloseSummary
import uniffi.gemstone.GemConfirmationInterface

object PerpetualModifyAutocloseFactory {

    fun create(
        data: PerpetualModifyConfirmData,
        session: GemConfirmationInterface,
    ): ConfirmDetailElement.PerpetualModifyAutoclose? =
        session.autocloseSummary(data)?.let(::element)

    internal fun element(summary: GemAutocloseSummary): ConfirmDetailElement.PerpetualModifyAutoclose =
        ConfirmDetailElement.PerpetualModifyAutoclose(
            takeProfitText = summary.takeProfit?.text()
                ?: Placeholder.empty.takeIf { summary.takeProfitCleared },
            stopLossText = summary.stopLoss?.text()
                ?: Placeholder.empty.takeIf { summary.stopLossCleared },
        )
}
