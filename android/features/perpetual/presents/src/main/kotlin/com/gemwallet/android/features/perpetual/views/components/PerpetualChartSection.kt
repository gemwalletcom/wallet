package com.gemwallet.android.features.perpetual.views.components

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
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.perpetual.viewmodels.models.PerpetualChartUIModel
import com.gemwallet.android.ui.components.chart.CandlestickTooltip
import com.gemwallet.android.ui.components.chart.CandlestickTooltipUIModel
import com.gemwallet.android.ui.components.chart.ChartStateView
import com.gemwallet.android.ui.components.chart.GemCandlestickChart
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.candlestickHeader
import uniffi.gemstone.chartDateStyle
import java.time.ZoneId
import java.util.Locale

private val TooltipRightSafeArea = 96.dp

@Composable
internal fun PerpetualChartSection(state: StateViewType<PerpetualChartUIModel>, period: ChartPeriod, tooltip: (ChartCandleStick) -> CandlestickTooltipUIModel, onPeriodSelect: (ChartPeriod) -> Unit, modifier: Modifier = Modifier) {
    val model = state.dataOrNull
    val data = model?.candles.orEmpty()
    var selectedIndex by remember(period) { mutableStateOf<Int?>(null) }
    val safeSelectedIndex = selectedIndex?.takeIf { it in data.indices }
    val selectedCandle = safeSelectedIndex?.let { data[it] }
    val baseCandle = data.firstOrNull()
    val lastCandle = data.lastOrNull()
    val isSelectedRightHalf = safeSelectedIndex?.let { it.toFloat() / data.size.toFloat() > 0.5f } ?: false

    val chartUIModel = model?.chart
    val header = remember(selectedCandle, baseCandle, lastCandle) {
        val target = selectedCandle ?: lastCandle ?: return@remember null
        val base = baseCandle ?: return@remember null
        candlestickHeader(base.close, target.close)
    }
    val dateFormatter = LocalContext.current.rowDateFormatter()
    val headerDate = remember(selectedCandle, period) {
        selectedCandle?.let { dateFormatter.chartDate(it.date, chartDateStyle(period.toGem()), ZoneId.systemDefault(), Locale.getDefault()) }
    }
    val tooltipModel = remember(selectedCandle) { selectedCandle?.let(tooltip) }

    ChartStateView(
        state = state,
        header = header,
        date = headerDate,
        period = period,
        onPeriodSelect = onPeriodSelect,
        modifier = modifier,
    ) { _ ->
        if (chartUIModel != null) {
            GemCandlestickChart(
                model = chartUIModel,
                selectedIndex = safeSelectedIndex,
                onSelectionChanged = { selectedIndex = it },
            )
            TooltipOverlay(
                visible = tooltipModel != null,
                tooltip = tooltipModel,
                alignToStart = isSelectedRightHalf,
            )
        }
    }
}

@Composable
private fun BoxScope.TooltipOverlay(visible: Boolean, tooltip: CandlestickTooltipUIModel?, alignToStart: Boolean) {
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
        tooltip?.let { CandlestickTooltip(model = it) }
    }
}
