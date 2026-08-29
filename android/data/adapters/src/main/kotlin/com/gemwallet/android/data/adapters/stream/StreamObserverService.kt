package com.gemwallet.android.data.adapters.stream

import android.util.Log
import com.gemwallet.android.application.assets.cases.SyncAssets
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import uniffi.gemstone.GemStreamService
import uniffi.gemstone.GemStreamSubscriptionService
import com.gemwallet.android.ext.runCatchingCancellable
import uniffi.gemstone.GemDeviceService

class StreamObserverService(
    private val getSession: GetSession,
    private val getCurrentCurrency: GetCurrentCurrency,
    private val syncAssets: SyncAssets,
    private val subscriptionService: GemStreamSubscriptionService,
    private val streamService: GemStreamService,
    private val connection: WebSocketConnectable,
    private val deviceService: GemDeviceService,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private var connectionJob: Job? = null
    private var currentWalletId: String? = null

    init {
        scope.launch {
            getSession().collectLatest { session ->
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
        if (getSession().value?.wallet == null) return
        connectionJob = scope.launch {
            runCatchingCancellable { deviceService.synchronizeIfNeeded() }
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
            runCatchingCancellable { streamService.handle(text, getCurrentCurrency.getCurrentCurrency().toJson()) }
                .onFailure { Log.e(TAG, "Event handler error", it) }
        }
    }

    companion object {
        private const val TAG = "StreamObserverService"
    }
}
