package com.gemwallet.android.ui.models

import androidx.compose.runtime.Immutable
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemChartStyle

@Immutable
data class ChartUIState(val period: ChartPeriod, val periods: List<ChartPeriod>, val phase: GemChartPhase = GemChartPhase.Loading, val styleSwitch: GemChartStyle? = null)
