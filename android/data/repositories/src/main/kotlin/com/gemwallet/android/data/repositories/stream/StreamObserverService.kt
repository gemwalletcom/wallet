package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.Constants
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.http.DeviceRequestSigner
import com.gemwallet.android.serializer.StreamEventSerializer
import com.gemwallet.android.serializer.jsonEncoder
import com.wallet.core.primitives.StreamMessage
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.wss
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.header
import io.ktor.http.HttpMethod
import io.ktor.websocket.Frame
import io.ktor.websocket.readText
import io.ktor.websocket.send
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.onSubscription
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString

class StreamObserverService(
    private val sessionRepository: SessionRepository,
    private val syncAssets: SyncAssets,
    private val deviceRequestSigner: DeviceRequestSigner,
    private val subscriptionService: StreamSubscriptionService,
    private val eventHandler: StreamEventHandler,
    private val reconnection: ExponentialReconnection = ExponentialReconnection(),
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) {
    private var connectionJob: Job? = null
    private var currentWalletId: String? = null
    private var reconnectAttempt = 0

    private val client = HttpClient(CIO) {
        install(WebSockets) {
            pingIntervalMillis = PING_INTERVAL_MS
        }
    }

    init {
        scope.launch {
            sessionRepository.session().collectLatest { session ->
                val walletId = session?.wallet?.id
                if (walletId != null && walletId != currentWalletId) {
                    currentWalletId = walletId
                    subscriptionService.setupAssets(walletId)
                    if (connectionJob == null) {
                        start()
                    }
                    try {
                        syncAssets()
                    } catch (cancelled: CancellationException) {
                        throw cancelled
                    } catch (_: Throwable) {
                    }
                }
            }
        }
    }

    fun start() {
        if (connectionJob != null) return

        connectionJob = scope.launch(Dispatchers.IO) {
            val hasWallet = sessionRepository.session().firstOrNull()?.wallet != null
            if (!hasWallet) {
                connectionJob = null
                return@launch
            }

            while (isActive) {
                try {
                    client.wss(
                        method = HttpMethod.Get,
                        host = Constants.API_HOST,
                        port = 443,
                        path = Constants.DEVICE_STREAM_PATH,
                        request = { addDeviceAuthHeaders() },
                    ) {
                        reconnectAttempt = 0
                        observeConnection()
                    }
                } catch (err: Throwable) {
                    Log.e(TAG, "Connection error", err)
                }
                delay(reconnection.reconnectAfterMs(reconnectAttempt))
                reconnectAttempt++
            }
        }
    }

    fun stop() {
        connectionJob?.cancel()
        connectionJob = null
        reconnectAttempt = 0
    }

    private suspend fun DefaultClientWebSocketSession.observeConnection() {
        launch {
            subscriptionService.messageFlow
                .onSubscription { subscriptionService.resubscribe() }
                .collect { message ->
                    try {
                        send(jsonEncoder.encodeToString<StreamMessage>(message))
                    } catch (err: Throwable) {
                        Log.e(TAG, "Send message error", err)
                    }
                }
        }
        for (frame in incoming) {
            if (frame is Frame.Text) {
                val text = frame.readText()
                try {
                    val event = jsonEncoder.decodeFromString(StreamEventSerializer, text)
                    scope.launch { eventHandler.handle(event) }
                } catch (err: Throwable) {
                    Log.e(TAG, "Parse event error: ${text.take(100)}", err)
                }
            }
        }
    }

    private fun HttpRequestBuilder.addDeviceAuthHeaders() {
        deviceRequestSigner.sign(HttpMethod.Get.value, Constants.DEVICE_STREAM_PATH)
            .toHeaders()
            .forEach { (key, value) -> header(key, value) }
    }

    companion object {
        private const val TAG = "StreamObserverService"
        private const val PING_INTERVAL_MS = 30_000L
    }
}
