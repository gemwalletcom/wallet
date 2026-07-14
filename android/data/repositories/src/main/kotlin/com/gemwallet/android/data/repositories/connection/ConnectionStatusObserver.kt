package com.gemwallet.android.data.repositories.connection

import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionComponentHealth
import com.wallet.core.primitives.ConnectionStatus
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

class ConnectionStatusObserver(
    private val monitors: List<ConnectionComponentMonitor>,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private val state = MutableStateFlow<Map<ConnectionComponent, ConnectionComponentHealth>>(emptyMap())

    val healthByComponent: StateFlow<Map<ConnectionComponent, ConnectionComponentHealth>> = state.asStateFlow()

    val status: StateFlow<ConnectionStatus> = state
        .map { it.rollup() }
        .stateIn(scope, SharingStarted.Eagerly, ConnectionStatus.Online)

    private var jobs: List<Job> = emptyList()

    fun start() {
        if (jobs.isNotEmpty()) return
        jobs = monitors.map { monitor ->
            scope.launch {
                monitor.healthFlow().collect { health ->
                    update(monitor.component, health)
                }
            }
        }
    }

    fun stop() {
        jobs.forEach { it.cancel() }
        jobs = emptyList()
    }

    internal fun update(component: ConnectionComponent, health: ConnectionComponentHealth) {
        state.update { current ->
            val isInternetRecovered = component == ConnectionComponent.Internet
                && health.isHealthy
                && current[ConnectionComponent.Internet]?.isHealthy == false
            val base = if (isInternetRecovered) emptyMap() else current
            base + (component to health)
        }
    }
}
