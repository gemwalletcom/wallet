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
import com.gemwallet.android.features.perpetuals.viewmodels.models.PerpetualChartUIModel
import com.gemwallet.android.ui.components.chart.CandleTooltipUIModel
import com.gemwallet.android.ui.components.chart.CandlestickTooltip
import com.gemwallet.android.ui.components.chart.ChartStateView
import com.gemwallet.android.ui.components.chart.GemCandlestickChart
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.gemwallet.android.ui.theme.paddingSmall
import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod

private val TooltipRightSafeArea = 96.dp

@Composable
internal fun PerpetualChartSection(state: StateViewType<PerpetualChartUIModel>, period: ChartPeriod, tooltip: (ChartCandleStick) -> CandleTooltipUIModel, onPeriodSelect: (ChartPeriod) -> Unit, modifier: Modifier = Modifier) {
    val model = state.dataOrNull
    val data = model?.candles.orEmpty()
    var selectedIndex by remember(period) { mutableStateOf<Int?>(null) }
    val safeSelectedIndex = selectedIndex?.takeIf { it in data.indices }
    val selectedCandle = safeSelectedIndex?.let { data[it] }
    val isSelectedRightHalf = safeSelectedIndex?.let { it.toFloat() / data.size.toFloat() > 0.5f } ?: false

    val chartUIModel = model?.chart
    val header = remember(model, safeSelectedIndex) { model?.header(safeSelectedIndex) }
    val dateFormatter = LocalContext.current.rowDateFormatter()
    val headerDate = remember(model, safeSelectedIndex, period) { model?.dateText(safeSelectedIndex, period, dateFormatter) }
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
private fun BoxScope.TooltipOverlay(visible: Boolean, tooltip: CandleTooltipUIModel?, alignToStart: Boolean) {
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
