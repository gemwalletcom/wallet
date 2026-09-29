package com.gemwallet.android.features.assets.presents.market

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.assets.viewmodels.market.ChartValuesViewModel
import com.gemwallet.android.ui.components.chart.ChartSection

@Composable
fun Chart(viewModel: ChartValuesViewModel = hiltViewModel()) {
    val state by viewModel.chartUIState.collectAsStateWithLifecycle()

    ChartSection(state = state, onPeriodSelect = viewModel::setPeriod, onZoom = viewModel::onZoom, onPan = viewModel::onPan)
}
