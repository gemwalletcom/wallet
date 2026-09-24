package com.gemwallet.android.data.services.gemstone.stream

import android.util.Log
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.connection.ConnectionComponentHealth
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemConnectionServiceInterface
import uniffi.gemstone.GemStreamServiceInterface
import kotlin.time.TimeMark
import kotlin.time.TimeSource

class StreamObserverService(
    private val getSession: GetSession,
    private val service: GemStreamServiceInterface,
    private val connection: WebSocketConnectable,
    private val health: ConnectionComponentHealth,
    private val connectionService: GemConnectionServiceInterface,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private var connectionJob: Job? = null

    @Synchronized
    fun start() {
        if (connectionJob?.isActive == true) return
        val previousJob = connectionJob
        connectionJob = scope.launch {
            withContext(NonCancellable) { previousJob?.join() }
            currentCoroutineContext().ensureActive()
            val wallets = getSession().map { it?.wallet?.id?.id }.distinctUntilChanged()
            launch {
                wallets.drop(1).collect {
                    runCatchingCancellable { service.updateSession() }
                        .onFailure { Log.e(TAG, "Stream session update error", it) }
                }
            }
            var failedAttempts = 0u
            while (isActive) {
                var subscribedAt: TimeMark? = null
                runCatchingCancellable {
                    val connects = service.prepareConnection()
                    currentCoroutineContext().ensureActive()
                    when (connects) {
                        true -> observeConnection(onSubscribed = { subscribedAt = TimeSource.Monotonic.markNow() })
                        false -> wallets.first { it != null }
                    }
                }.onFailure {
                    Log.e(TAG, "Stream connection error", it)
                    val reconnection = connectionService.reconnection(failedAttempts, subscribedAt.connectedDuration())
                    failedAttempts = reconnection.nextAttempt
                    delay(reconnection.delay.toMillis())
                }
            }
        }
    }

    @Synchronized
    fun stop() {
        connectionJob?.cancel()
    }

    private suspend fun observeConnection(onSubscribed: () -> Unit) {
        try {
            connection.connect().collect { event ->
                when (event) {
                    WebSocketEvent.Connected -> {
                        service.connected()
                        health.report(isHealthy = true)
                        onSubscribed()
                    }

                    is WebSocketEvent.Message -> runCatchingCancellable {
                        val handled = service.decodeEvent(event.text)
                        scope.launch {
                            runCatchingCancellable { service.sync(handled) }
                                .onFailure { Log.e(TAG, "Stream sync error", it) }
                        }
                    }.onFailure { Log.e(TAG, "Stream event error", it) }

                    WebSocketEvent.Disconnected -> {
                        health.report(isHealthy = false)
                        runCatchingCancellable { service.disconnected() }
                            .onFailure { Log.e(TAG, "Stream event error", it) }
                    }
                }
            }
        } finally {
            health.report(isHealthy = false)
            withContext(NonCancellable) {
                runCatchingCancellable { service.disconnected() }
                    .onFailure { Log.e(TAG, "Stream disconnect error", it) }
            }
        }
    }

    companion object {
        private const val TAG = "StreamObserverService"
    }
}
