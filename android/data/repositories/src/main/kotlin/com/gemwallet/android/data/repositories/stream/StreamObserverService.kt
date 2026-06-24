package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.cases.device.IsDeviceRegistered
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.hasPerpetualsSupport
import com.gemwallet.android.model.Session
import com.gemwallet.android.serializer.StreamEventSerializer
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.StreamMessage
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString

class StreamObserverService(
    private val sessionRepository: SessionRepository,
    private val userConfig: UserConfig,
    private val syncAssets: SyncAssets,
    private val syncPerpetuals: SyncPerpetuals,
    private val subscriptionService: StreamSubscriptionService,
    private val eventHandler: StreamEventHandler,
    private val connection: WebSocketConnectable,
    private val syncDeviceInfo: SyncDeviceInfo,
    private val isDeviceRegistered: IsDeviceRegistered,
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
        scope.launchPerpetualSync(
            session = sessionRepository.session(),
            isPerpetualEnabled = userConfig.isPerpetualEnabled(),
            syncPerpetuals = syncPerpetuals,
        )
    }

    fun start() {
        if (connectionJob != null) return
        if (sessionRepository.session().value?.wallet == null) return

        connectionJob = scope.launch {
            if (!isDeviceRegistered.isDeviceRegistered()) {
                registerDevice()
            }
            launch {
                for (message in subscriptionService.messages) {
                    connection.send(jsonEncoder.encodeToString<StreamMessage>(message))
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

    private suspend fun registerDevice() {
        runCatching { syncDeviceInfo.syncDeviceInfo() }
            .onFailure { Log.e(TAG, "Device registration error", it) }
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

internal fun CoroutineScope.launchPerpetualSync(
    session: Flow<Session?>,
    isPerpetualEnabled: Flow<Boolean>,
    syncPerpetuals: SyncPerpetuals,
): Job = launch {
    combine(session, isPerpetualEnabled) { current, enabled ->
        val wallet = current?.wallet
        if (wallet != null && wallet.hasPerpetualsSupport && enabled) wallet.id.id else null
    }
        .distinctUntilChanged()
        .collectLatest { walletId ->
            if (walletId == null) return@collectLatest
            runCatching { syncPerpetuals.syncPerpetuals() }
        }
}
