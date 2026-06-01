package com.gemwallet.android.features.perpetual.views.autoclose

import com.wallet.core.primitives.TpslType

internal sealed interface AutocloseAction {
    data object Close : AutocloseAction
    data object Confirm : AutocloseAction
    data class TakeProfitChanged(val text: String) : AutocloseAction
    data class StopLossChanged(val text: String) : AutocloseAction
    data class SelectPercent(val type: TpslType, val percent: Int) : AutocloseAction
}
