package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.ui.models.perpetual.PerpetualConfirmDetailsUIModel
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel

sealed interface ConfirmDetailElement {
    data class SwapDetails(
        val model: SwapDetailsUIModel,
    ) : ConfirmDetailElement

    data class PerpetualDetails(
        val model: PerpetualConfirmDetailsUIModel,
    ) : ConfirmDetailElement

    data class PerpetualModifyAutoclose(
        val takeProfitText: String?,
        val stopLossText: String?,
    ) : ConfirmDetailElement
}
