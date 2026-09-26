package com.gemwallet.android.features.market.presents

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.market.viewmodels.ChartValuesViewModel
import com.gemwallet.android.features.market.viewmodels.models.ChartUIState
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.components.chart.ChartStateView
import com.gemwallet.android.ui.components.chart.GemLineChart
import com.gemwallet.android.ui.format.rowDateFormatter
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.ChartPeriod
import java.time.ZoneId
import java.util.Locale

@Composable
fun Chart(viewModel: ChartValuesViewModel = hiltViewModel()) {
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()

    ChartSection(state = state, onPeriodSelect = viewModel::setPeriod)
}

@Composable
fun ChartSection(state: ChartUIState, onPeriodSelect: (ChartPeriod) -> Unit, periods: List<ChartPeriod> = ChartPeriod.entries) {
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
            val minLabel = remember(model) { model.bounds.low.text() }
            val maxLabel = remember(model) { model.bounds.high.text() }
            val points = remember(model) { model.values.mapIndexed { index, value -> ChartPoint(x = index.toFloat(), y = value.value.toFloat()) } }
            GemLineChart(
                points = points,
                bounds = model.bounds,
                lineColor = MaterialTheme.colorScheme.primary,
                selectedIndex = selectedIndex,
                onSelectionChanged = { selectedIndex = it },
                minLabel = minLabel,
                maxLabel = maxLabel,
            )
        }
    }
}
