package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemPerpetualConfirmDetails

sealed interface ConfirmDetailsUIModel {
    data class SwapDetails(val model: SwapDetailsUIModel) : ConfirmDetailsUIModel

    data class PerpetualDetails(val details: GemPerpetualConfirmDetails) : ConfirmDetailsUIModel

    data class PerpetualModifyAutoclose(val row: GemListRow) : ConfirmDetailsUIModel
}
