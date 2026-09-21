package com.gemwallet.android.data.services.gemstone.connection

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionStatus
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemConnectionServiceInterface
import uniffi.gemstone.GemRefreshKind

class ConnectionStatusObserver(private val monitors: List<ConnectionComponentMonitor>, private val connectionService: GemConnectionServiceInterface, private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO)) {
    private val state = MutableStateFlow<Map<ConnectionComponent, Boolean>>(emptyMap())

    val isHealthyByComponent: StateFlow<Map<ConnectionComponent, Boolean>> = state.asStateFlow()

    val status: StateFlow<ConnectionStatus> = state
        .map { it.connectionStatus }
        .distinctUntilChanged()
        .stateIn(scope, SharingStarted.Eagerly, ConnectionStatus.Online)

    private var jobs: List<Job> = emptyList()

    fun refreshIntervalMillis(kind: GemRefreshKind): Flow<Long> = status.map { connectionService.refreshInterval(kind, it.toGem()).toMillis() }

    fun start() {
        if (jobs.isNotEmpty()) return
        jobs = monitors.map { monitor ->
            scope.launch {
                monitor.healthFlow().collect { isHealthy ->
                    update(monitor.component, isHealthy)
                }
            }
        }
    }

    fun stop() {
        jobs.forEach { it.cancel() }
        jobs = emptyList()
    }

    internal fun update(component: ConnectionComponent, isHealthy: Boolean) {
        state.update { current ->
            val base = if (connectionService.resetsComponentHealth(component.toGem(), isHealthy, current[component])) emptyMap() else current
            base + (component to isHealthy)
        }
    }
}
