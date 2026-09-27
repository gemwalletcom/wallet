package com.gemwallet.android.features.perpetuals.presents.perpetual

import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TransactionId

internal sealed interface PerpetualAction {
    data object Close : PerpetualAction
    data object Refresh : PerpetualAction
    data object IncreasePosition : PerpetualAction
    data object ReducePosition : PerpetualAction
    data object ClosePosition : PerpetualAction
    data object Autoclose : PerpetualAction
    data class OpenPosition(val direction: PerpetualDirection) : PerpetualAction
    data class SelectChartPeriod(val period: ChartPeriod) : PerpetualAction
    data class OpenTransaction(val transactionId: TransactionId) : PerpetualAction
}
