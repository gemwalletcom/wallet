package com.gemwallet.android.data.services.gemstone.stream

import android.util.Log
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemStreamServiceInterface

class StreamObserverService(
    private val getSession: GetSession,
    private val service: GemStreamServiceInterface,
    private val connection: WebSocketConnectable,
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
            while (isActive) {
                runCatchingCancellable {
                    val connects = service.prepareConnection()
                    currentCoroutineContext().ensureActive()
                    when (connects) {
                        true -> observeConnection()
                        false -> wallets.first { it != null }
                    }
                }.onFailure { Log.e(TAG, "Stream connection error", it) }
            }
        }
    }

    @Synchronized
    fun stop() {
        connectionJob?.cancel()
    }

    private suspend fun observeConnection() {
        try {
            connection.connect().collect { event ->
                runCatchingCancellable {
                    when (event) {
                        WebSocketEvent.Connected -> service.connected()
                        is WebSocketEvent.Message -> Log.d(TAG, "Stream event: ${service.handle(event.text)}")
                        WebSocketEvent.Disconnected -> service.disconnected()
                    }
                }.onFailure { Log.e(TAG, "Stream event error", it) }
            }
        } finally {
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
