package com.gemwallet.android.ui.models

import androidx.compose.runtime.Immutable
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemCandleChart
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartStyle

@Immutable
data class ChartUIState(
    val period: ChartPeriod = ChartPeriod.Day,
    val chart: StateViewType<GemChartData> = StateViewType.Loading,
    val candles: GemCandleChart? = null,
    val styleSwitch: GemChartStyle? = null,
    val periods: List<ChartPeriod> = ChartPeriod.entries,
)
