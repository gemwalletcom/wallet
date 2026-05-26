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
import com.gemwallet.android.features.asset.viewmodels.chart.models.PricePoint
import com.gemwallet.android.features.asset.viewmodels.chart.models.chartHeader
import com.gemwallet.android.features.asset.viewmodels.chart.viewmodels.ChartViewModel
import com.gemwallet.android.ui.components.chart.ChartStateView
import com.gemwallet.android.ui.components.chart.GemLineChart
import com.gemwallet.android.ui.models.chart.ChartViewState

@Composable
fun Chart(viewModel: ChartViewModel = hiltViewModel()) {
    val uiModel by viewModel.chartUIModel.collectAsStateWithLifecycle()
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()

    key(state.period) {
        var selectedIndex by remember { mutableStateOf<Int?>(null) }

        val displayState = when {
            state.period != uiModel.period -> ChartViewState.Loading
            else -> state.viewState
        }
        val chartPoints = uiModel.chartPoints
        val selectedPoint: PricePoint? = if (displayState == ChartViewState.Ready) {
            selectedIndex?.let { chartPoints.getOrNull(it) }
        } else {
            null
        }

        ChartStateView(
            state = displayState,
            header = chartHeader(uiModel, selectedPoint),
            period = state.period,
            onPeriodSelect = viewModel::setPeriod,
        ) {
            GemLineChart(
                points = uiModel.renderPoints,
                lineColor = MaterialTheme.colorScheme.primary,
                selectedIndex = selectedIndex,
                onSelectionChanged = { selectedIndex = it },
                minLabel = uiModel.minLabel,
                maxLabel = uiModel.maxLabel,
            )
        }
    }
}
