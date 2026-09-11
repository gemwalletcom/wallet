package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.ChartViewModel
import com.gemwallet.android.ui.components.chart.ChartStateView
import com.gemwallet.android.ui.components.chart.GemLineChart
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.ChartPeriod

@Composable
fun Chart(viewModel: ChartViewModel = hiltViewModel()) {
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()

    ChartSection(state = state, onPeriodSelect = viewModel::setPeriod)
}

@Composable
internal fun ChartSection(
    state: ChartUIModel.State,
    onPeriodSelect: (ChartPeriod) -> Unit,
    periods: List<ChartPeriod> = ChartPeriod.entries,
) {
    key(state.period) {
        var selectedIndex by remember { mutableStateOf<Int?>(null) }

        val uiModel = state.chart.dataOrNull

        ChartStateView(
            state = state.chart,
            header = uiModel?.header(selectedIndex),
            period = state.period,
            onPeriodSelect = onPeriodSelect,
            periods = periods,
        ) { model ->
            GemLineChart(
                points = model.renderPoints,
                lineColor = MaterialTheme.colorScheme.primary,
                selectedIndex = selectedIndex,
                onSelectionChanged = { selectedIndex = it },
                minLabel = model.minLabel,
                maxLabel = model.maxLabel,
            )
        }
    }
}
