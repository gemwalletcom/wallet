package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import uniffi.gemstone.GemStreamService
import uniffi.gemstone.GemStreamSubscriptionService
import com.gemwallet.android.ext.runCatchingCancellable

class StreamObserverService(
    private val sessionRepository: SessionRepository,
    private val syncAssets: SyncAssets,
    private val subscriptionService: GemStreamSubscriptionService,
    private val streamService: GemStreamService,
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
                runCatchingCancellable { subscriptionService.setupAssets(wallet.id.id) }
                    .onFailure { Log.e(TAG, "Setup assets error", it) }
                if (connectionJob == null) start()
                runCatchingCancellable { syncAssets() }
                    .onFailure { Log.e(TAG, "Assets synchronization error", it) }
            }
        }
    }

    fun start() {
        if (connectionJob != null) return
        if (sessionRepository.session().value?.wallet == null) return
        connectionJob = scope.launch {
            runCatchingCancellable { syncDevice.syncDevice() }
                .onFailure { Log.e(TAG, "Device synchronization error", it) }
            connection.connect().collect { event ->
                when (event) {
                    WebSocketEvent.Connected -> runCatchingCancellable { subscriptionService.resubscribe() }
                        .onFailure { Log.e(TAG, "Resubscribe error", it) }
                    is WebSocketEvent.Message -> handleMessage(event.text)
                    WebSocketEvent.Disconnected -> subscriptionService.reset()
                }
            }
        }
    }

    fun stop() {
        connectionJob?.cancel()
        connectionJob = null
    }

    private fun handleMessage(text: String) {
        scope.launch {
            runCatchingCancellable { streamService.handle(text, sessionRepository.getCurrentCurrency().toJson()) }
                .onFailure { Log.e(TAG, "Event handler error", it) }
        }
    }

    companion object {
        private const val TAG = "StreamObserverService"
    }
}
