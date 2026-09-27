package com.gemwallet.android.features.market.viewmodels.models

import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartData

internal const val StopTimeoutMillis = 5_000L

data class ChartUIState(val period: ChartPeriod = ChartPeriod.Day, val chart: StateViewType<GemChartData> = StateViewType.Loading)
