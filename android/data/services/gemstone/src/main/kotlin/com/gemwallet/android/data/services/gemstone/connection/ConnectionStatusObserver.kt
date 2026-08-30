package com.gemwallet.android.data.services.gemstone.connection

import android.util.Log
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.ConnectionStatus
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemConnectionService

class ConnectionStatusObserver(
    private val monitors: List<ConnectionComponentMonitor>,
    private val connectionService: GemConnectionService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private val state = MutableStateFlow<Map<ConnectionComponent, Boolean>>(emptyMap())

    val isHealthyByComponent: StateFlow<Map<ConnectionComponent, Boolean>> = state.asStateFlow()

    val status: StateFlow<ConnectionStatus> = state
        .map { it.connectionStatus }
        .distinctUntilChanged()
        .onEach { Log.d(TAG, "Connection status changed: $it") }
        .stateIn(scope, SharingStarted.Eagerly, ConnectionStatus.Online)

    private var jobs: List<Job> = emptyList()

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
            val base = if (connectionService.resetsComponentHealth(component.toJson(), isHealthy, current[component])) emptyMap() else current
            base + (component to isHealthy)
        }
    }

    private companion object {
        const val TAG = "ConnectionStatusObserver"
    }
}
