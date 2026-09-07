package com.gemwallet.android.data.services.gemstone.stream

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
import uniffi.gemstone.GemConnectionService
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
    private val connectionService: GemConnectionService,
) : WebSocketConnectable {
    private val client = client.newBuilder()
        .pingInterval(connectionService.pingIntervalMilliseconds().toLong(), TimeUnit.MILLISECONDS)
        .build()
    private val activeSession = AtomicReference<WebSocketSession?>()

    override val isConnected: Boolean
        get() = activeSession.get()?.webSocket?.get() != null

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
            delay(connectionService.reconnectDelayMilliseconds(reconnectAttempt.toUInt()).toLong())
            reconnectAttempt++
        }
    }

    override suspend fun send(message: String): Boolean = activeSession.get()?.webSocket?.get()?.send(message) == true

    private fun observeSession(request: WebSocketRequest): Flow<WebSocketEvent> = callbackFlow {
        val session = WebSocketSession()
        activeSession.set(session)
        val webSocket = client.newWebSocket(request.toOkHttpRequest(), object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {
                if (activeSession.get() !== session) {
                    webSocket.cancel()
                    return
                }
                session.webSocket.set(webSocket)
                if (trySend(WebSocketEvent.Connected).isFailure) {
                    activeSession.compareAndSet(session, null)
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
                activeSession.compareAndSet(session, null)
                close()
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                activeSession.compareAndSet(session, null)
                close(t)
            }
        })
        awaitClose {
            activeSession.compareAndSet(session, null)
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

    private class WebSocketSession {
        val webSocket = AtomicReference<WebSocket?>()
    }

    companion object {
        private const val TAG = "WebSocketConnection"
    }
}
