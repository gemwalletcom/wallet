package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.serializer.StreamEventSerializer
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

class StreamObserverService(
    private val sessionRepository: SessionRepository,
    private val syncAssets: SyncAssets,
    private val subscriptionService: StreamSubscriptionService,
    private val eventHandler: StreamEventHandler,
    private val connection: WebSocketConnectable,
    private val syncDevice: SyncDevice,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private var connectionJob: Job? = null
    private var currentWalletId: String? = null

    init {
        scope.launch {
            sessionRepository.session().collectLatest { session ->
                val wallet = session?.wallet ?: return@collectLatest
                if (wallet.id.id == currentWalletId) return@collectLatest
                currentWalletId = wallet.id.id
                subscriptionService.setupAssets(wallet.id)
                if (connectionJob == null) start()
                runCatching { syncAssets() }
            }
        }
    }

    fun start() {
        if (connectionJob != null) return
        if (sessionRepository.session().value?.wallet == null) return

        connectionJob = scope.launch {
            runCatching { syncDevice.syncDevice() }
                .onFailure { Log.e(TAG, "Device synchronization error", it) }
            launch {
                for (message in subscriptionService.messages) {
                    connection.send(message.toJson())
                }
            }
            connection.connect().collect { event ->
                when (event) {
                    WebSocketEvent.Connected -> subscriptionService.resubscribe()
                    is WebSocketEvent.Message -> handleMessage(event.text)
                    WebSocketEvent.Disconnected -> Unit
                }
            }
        }
    }

    fun stop() {
        connectionJob?.cancel()
        connectionJob = null
    }

    private fun handleMessage(text: String) {
        try {
            val event = jsonEncoder.decodeFromString(StreamEventSerializer, text)
            scope.launch { eventHandler.handle(event) }
        } catch (err: Throwable) {
            Log.e(TAG, "Parse event error: ${text.take(100)}", err)
        }
    }

    companion object {
        private const val TAG = "StreamObserverService"
    }
}
