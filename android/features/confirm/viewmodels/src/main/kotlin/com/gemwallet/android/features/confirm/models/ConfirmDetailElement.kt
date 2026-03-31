package com.gemwallet.android.features.confirm.models

import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel

sealed interface ConfirmDetailElement {
    data class SwapDetails(
        val model: SwapDetailsUIModel,
    ) : ConfirmDetailElement
}
