package com.gemwallet.android.features.perpetuals.presents.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.components.chart.CandlestickTooltip
import com.gemwallet.android.ui.components.chart.ChartStateView
import com.gemwallet.android.ui.components.chart.GemCandlestickChart
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemCandleChart
import uniffi.gemstone.GemCandleTooltip
import java.time.ZoneId
import java.util.Locale

private val TooltipRightSafeArea = 96.dp

@Composable
internal fun PerpetualChartSection(state: StateViewType<GemCandleChart>, period: ChartPeriod, onPeriodSelect: (ChartPeriod) -> Unit, modifier: Modifier = Modifier) {
    val chart = state.dataOrNull
    val data = chart?.candles.orEmpty()
    var selectedIndex by remember(period) { mutableStateOf<Int?>(null) }
    val safeSelectedIndex = selectedIndex?.takeIf { it in data.indices }
    val isSelectedRightHalf = safeSelectedIndex?.let { it.toFloat() / data.size.toFloat() > 0.5f } ?: false

    val selection = remember(chart, safeSelectedIndex) { safeSelectedIndex?.let { chart?.selection(it.toUInt()) } }
    val dateFormatter = LocalContext.current.rowDateFormatter()
    val headerDate = chart?.let { data -> selection?.let { dateFormatter.chartDate(it.date, data.dateStyle, ZoneId.systemDefault(), Locale.getDefault()) } }
    val tooltip = remember(chart, safeSelectedIndex) { safeSelectedIndex?.let { chart?.tooltip(it.toUInt()) } }

    ChartStateView(
        state = state,
        header = selection?.header ?: chart?.header,
        date = headerDate,
        period = period,
        onPeriodSelect = onPeriodSelect,
        modifier = modifier,
    ) { _ ->
        if (chart != null) {
            GemCandlestickChart(
                chart = chart,
                selectedIndex = safeSelectedIndex,
                onSelectionChanged = { selectedIndex = it },
            )
            TooltipOverlay(
                visible = tooltip != null,
                tooltip = tooltip,
                alignToStart = isSelectedRightHalf,
            )
        }
    }
}

@Composable
private fun BoxScope.TooltipOverlay(visible: Boolean, tooltip: GemCandleTooltip?, alignToStart: Boolean) {
    AnimatedVisibility(
        visible = visible,
        enter = fadeIn(),
        exit = fadeOut(),
        modifier = Modifier
            .align(if (alignToStart) Alignment.TopStart else Alignment.TopEnd)
            .padding(
                start = paddingSmall,
                end = if (alignToStart) paddingSmall else TooltipRightSafeArea,
                top = paddingSmall,
            ),
    ) {
        tooltip?.let { CandlestickTooltip(tooltip = it) }
    }
}
