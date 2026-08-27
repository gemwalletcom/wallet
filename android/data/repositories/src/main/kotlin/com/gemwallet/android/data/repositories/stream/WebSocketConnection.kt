package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.isActive
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference

data class WebSocketRequest(
    val url: String,
    val headers: Map<String, String> = emptyMap(),
)

sealed interface WebSocketEvent {
    data object Connected : WebSocketEvent
    data class Message(val text: String) : WebSocketEvent
    data object Disconnected : WebSocketEvent
}

interface WebSocketConnectable {
    val isConnected: Boolean

    fun connect(): Flow<WebSocketEvent>
    suspend fun send(message: String): Boolean
}

class WebSocketConnection(
    private val requestProvider: suspend () -> WebSocketRequest,
    client: OkHttpClient,
    private val reconnection: ExponentialReconnection = ExponentialReconnection(),
    private val pingInterval: Long = PING_INTERVAL_MS,
) : WebSocketConnectable {
    private val client = client.newBuilder()
        .pingInterval(pingInterval, TimeUnit.MILLISECONDS)
        .build()
    private val activeWebSocket = AtomicReference<WebSocket?>()

    override val isConnected: Boolean
        get() = activeWebSocket.get() != null

    override fun connect(): Flow<WebSocketEvent> = channelFlow {
        var reconnectAttempt = 0
        while (isActive) {
            runCatchingCancellable {
                observeSession(requestProvider()).collect { event ->
                    if (event == WebSocketEvent.Connected) reconnectAttempt = 0
                    send(event)
                }
            }.onFailure { Log.e(TAG, "Connection error", it) }
            send(WebSocketEvent.Disconnected)
            delay(reconnection.reconnectAfterMs(reconnectAttempt))
            reconnectAttempt++
        }
    }

    override suspend fun send(message: String): Boolean = activeWebSocket.get()?.send(message) == true

    private fun observeSession(request: WebSocketRequest): Flow<WebSocketEvent> = callbackFlow {
        val webSocket = client.newWebSocket(request.toOkHttpRequest(), object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                activeWebSocket.set(webSocket)
                if (trySend(WebSocketEvent.Connected).isFailure) {
                    activeWebSocket.compareAndSet(webSocket, null)
                    webSocket.cancel()
                }
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                trySend(WebSocketEvent.Message(text))
            }

            override fun onClosing(webSocket: WebSocket, code: Int, reason: String) {
                webSocket.close(code, reason)
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                activeWebSocket.compareAndSet(webSocket, null)
                close()
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                activeWebSocket.compareAndSet(webSocket, null)
                close(t)
            }
        })
        awaitClose {
            activeWebSocket.compareAndSet(webSocket, null)
            webSocket.cancel()
        }
    }

    private fun WebSocketRequest.toOkHttpRequest(): Request =
        Request.Builder()
            .url(url)
            .apply {
                headers.forEach { (name, value) -> header(name, value) }
            }
            .build()

    companion object {
        private const val TAG = "WebSocketConnection"
        private const val PING_INTERVAL_MS = 30_000L
    }
}
