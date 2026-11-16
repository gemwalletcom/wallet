package com.gemwallet.android.ui.components.chart

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ChartUIState
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartPhase
import java.time.ZoneId

@Composable
fun ChartSection(state: ChartUIState, onPeriodSelect: (ChartPeriod) -> Unit, onZoom: (Float, Float) -> Unit, onPan: (Float) -> Unit) {
    val context = LocalContext.current
    val lineState: StateViewType<GemChartData> = when (val phase = state.phase) {
        GemChartPhase.Loading -> StateViewType.Loading

        is GemChartPhase.Data -> StateViewType.Data(phase.data)

        is GemChartPhase.Candles -> return CandleChartSection(
            state = StateViewType.Data(phase.chart),
            period = state.period,
            onPeriodSelect = onPeriodSelect,
            onZoom = onZoom,
            onPan = onPan,
            periods = state.periods,
        )

        GemChartPhase.NoData -> StateViewType.NoData

        is GemChartPhase.Failed -> StateViewType.Error(phase.error.errorText().text(context))
    }
    LineChartSection(lineState, state.period, onPeriodSelect, onZoom, onPan, state.periods)
}

@Composable
private fun LineChartSection(state: StateViewType<GemChartData>, period: ChartPeriod, onPeriodSelect: (ChartPeriod) -> Unit, onZoom: (Float, Float) -> Unit, onPan: (Float) -> Unit, periods: List<ChartPeriod>) {
    key(period) {
        var selectedIndex by remember { mutableStateOf<Int?>(null) }
        val context = LocalContext.current
        val dateFormatter = remember(context) { context.rowDateFormatter() }

        val chart = state.dataOrNull
        val selection = remember(chart, selectedIndex) { selectedIndex?.let { chart?.selection(it) } }
        val date = chart?.let { data -> selection?.let { dateFormatter.chartDate(it.date, data.dateStyle, ZoneId.systemDefault(), LocalConfiguration.current.locales[0]) } }

        ChartStateView(
            state = state,
            header = selection?.header ?: chart?.header,
            date = date,
            period = period,
            onPeriodSelect = onPeriodSelect,
            periods = periods,
        ) { model ->
            val points = remember(model) { model.linePoints() }
            val minLabel = remember(model) { model.bounds.low.text() }
            val maxLabel = remember(model) { model.bounds.high.text() }
            GemLineChart(
                points = points,
                bounds = model.bounds,
                isZoomed = model.isZoomed,
                lineColor = MaterialTheme.colorScheme.primary,
                indexAt = { fraction -> model.indexAt(fraction.toDouble()) },
                selectedIndex = selectedIndex,
                onSelectionChanged = { selectedIndex = it },
                onZoom = onZoom,
                onPan = onPan,
                minLabel = minLabel,
                maxLabel = maxLabel,
            )
        }
    }
}

internal fun GemChartData.linePoints(): List<ChartPoint> = values.map { value -> ChartPoint(x = (value.date - start) / (end - start).coerceAtLeast(1L).toFloat(), y = value.value.toFloat()) }
