package com.gemwallet.android.ui.models.chart

sealed interface ChartViewState {
    data object Loading : ChartViewState
    data object Empty : ChartViewState
    data object Error : ChartViewState
    data object Ready : ChartViewState
}
