package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class ChartRefreshController {
    private val refreshTrigger = MutableStateFlow(0L)
    private val refreshState = MutableStateFlow(false)

    val trigger: StateFlow<Long> = refreshTrigger.asStateFlow()
    val isRefreshing: StateFlow<Boolean> = refreshState.asStateFlow()

    fun startRefreshing() {
        refreshState.value = true
        refreshTrigger.value = refreshTrigger.value + 1
    }

    fun stopRefreshing() {
        refreshState.value = false
    }
}
