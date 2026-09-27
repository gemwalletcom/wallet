package com.gemwallet.android.ui.models

import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartData

data class ChartUIState(val period: ChartPeriod = ChartPeriod.Day, val chart: StateViewType<GemChartData> = StateViewType.Loading)
