package com.gemwallet.android.features.perpetual.views.position

import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TransactionId

internal sealed interface PerpetualDetailsAction {
    data object Close : PerpetualDetailsAction
    data object Refresh : PerpetualDetailsAction
    data object IncreasePosition : PerpetualDetailsAction
    data object ReducePosition : PerpetualDetailsAction
    data object ClosePosition : PerpetualDetailsAction
    data object Autoclose : PerpetualDetailsAction
    data class OpenPosition(val direction: PerpetualDirection) : PerpetualDetailsAction
    data class SelectChartPeriod(val period: ChartPeriod) : PerpetualDetailsAction
    data class OpenTransaction(val transactionId: TransactionId) : PerpetualDetailsAction
}
