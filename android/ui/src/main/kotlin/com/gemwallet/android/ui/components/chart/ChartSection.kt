package com.gemwallet.android.ui.components.chart

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.models.ChartUIState
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartData
import java.time.ZoneId
import java.util.Locale

@Composable
fun ChartSection(state: ChartUIState, onPeriodSelect: (ChartPeriod) -> Unit, onZoom: (Float) -> Unit, periods: List<ChartPeriod> = ChartPeriod.entries) {
    key(state.period) {
        var selectedIndex by remember { mutableStateOf<Int?>(null) }
        val context = LocalContext.current
        val dateFormatter = remember(context) { context.rowDateFormatter() }

        val chart = state.chart.dataOrNull
        val selection = remember(chart, selectedIndex) { selectedIndex?.let { chart?.selection(it.toUInt()) } }
        val date = chart?.let { data -> selection?.let { dateFormatter.chartDate(it.date, data.dateStyle, ZoneId.systemDefault(), Locale.getDefault()) } }

        ChartStateView(
            state = state.chart,
            header = selection?.header ?: chart?.header,
            date = date,
            period = state.period,
            onPeriodSelect = onPeriodSelect,
            periods = periods,
        ) { model ->
            val points = remember(model) { model.linePoints() }
            val selectableFrom = remember(model) { model.selectableFrom() }
            val minLabel = remember(model) { model.boundLabel(model.bounds.lowerIndex, model.bounds.low.text()) }
            val maxLabel = remember(model) { model.boundLabel(model.bounds.upperIndex, model.bounds.high.text()) }
            GemLineChart(
                points = points,
                bounds = model.bounds,
                lineColor = MaterialTheme.colorScheme.primary,
                selectableFrom = selectableFrom,
                selectedIndex = selectedIndex,
                onSelectionChanged = { selectedIndex = it },
                onZoom = onZoom,
                minLabel = minLabel,
                maxLabel = maxLabel,
            )
        }
    }
}

internal fun GemChartData.linePoints(): List<ChartPoint> = values.map { value -> ChartPoint(x = fraction(value.date), y = value.value.toFloat()) }

internal fun GemChartData.selectableFrom(): Int = values.indexOfFirst { it.date >= start }.coerceAtLeast(0)

internal fun GemChartData.boundLabel(index: UInt, text: String): ChartBoundLabel? = values.getOrNull(index.toInt())?.let { ChartBoundLabel(x = fraction(it.date), text = text) }

private fun GemChartData.fraction(date: Long): Float = (date - start) / (end - start).coerceAtLeast(1L).toFloat()
