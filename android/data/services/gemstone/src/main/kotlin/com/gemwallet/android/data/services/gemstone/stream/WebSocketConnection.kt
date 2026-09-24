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
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withTimeoutOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import uniffi.gemstone.GemConnectionServiceInterface
import java.time.Duration
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import kotlin.coroutines.resume
import kotlin.time.TimeMark
import kotlin.time.TimeSource
import kotlin.time.toJavaDuration

data class WebSocketRequest(val url: String, val headers: Map<String, String> = emptyMap())

sealed interface WebSocketEvent {
    data object Connected : WebSocketEvent
    data class Message(val text: String) : WebSocketEvent
    data object Disconnected : WebSocketEvent
}

interface WebSocketConnectable {
    val isConnected: Boolean
    val connectionLatency: Duration?

    suspend fun ping(): Duration? = connectionLatency

    fun connect(): Flow<WebSocketEvent>
    suspend fun send(message: String): Boolean
}

class WebSocketConnection(private val requestProvider: suspend () -> WebSocketRequest, client: OkHttpClient, private val connectionService: GemConnectionServiceInterface) : WebSocketConnectable {
    private val client = client.newBuilder()
        .pingInterval(connectionService.pingIntervalMilliseconds().toLong(), TimeUnit.MILLISECONDS)
        .build()
    private val activeSession = AtomicReference<WebSocketSession?>()

    override val isConnected: Boolean
        get() = activeSession.get()?.webSocket?.get() != null

    override val connectionLatency: Duration?
        get() = activeSession.get()?.connectionLatency?.get()

    override suspend fun ping(): Duration? {
        val request = requestProvider()
        return withTimeoutOrNull(PROBE_TIMEOUT.toMillis()) {
            suspendCancellableCoroutine { continuation ->
                val socket = client.newWebSocket(
                    request.toOkHttpRequest(),
                    object : WebSocketListener() {
                        override fun onOpen(webSocket: WebSocket, response: Response) {
                            val elapsed = (response.receivedResponseAtMillis - response.sentRequestAtMillis).takeIf { it >= 0 }
                            webSocket.cancel()
                            if (continuation.isActive) continuation.resume(elapsed?.let(Duration::ofMillis))
                        }

                        override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {
                            if (continuation.isActive) continuation.resume(null)
                        }
                    },
                )
                continuation.invokeOnCancellation { socket.cancel() }
            }
        }
    }

    override fun connect(): Flow<WebSocketEvent> = channelFlow {
        var reconnectAttempt = 0u
        while (isActive) {
            var connectedAt: TimeMark? = null
            runCatchingCancellable {
                observeSession(requestProvider()).collect { event ->
                    if (event == WebSocketEvent.Connected) connectedAt = TimeSource.Monotonic.markNow()
                    send(event)
                }
            }.onFailure { Log.e(TAG, "Connection error", it) }
            send(WebSocketEvent.Disconnected)
            val reconnection = connectionService.reconnection(reconnectAttempt, connectedAt.connectedDuration())
            reconnectAttempt = reconnection.nextAttempt
            delay(reconnection.delay.toMillis())
        }
    }

    override suspend fun send(message: String): Boolean = activeSession.get()?.webSocket?.get()?.send(message) == true

    private fun observeSession(request: WebSocketRequest): Flow<WebSocketEvent> = callbackFlow {
        val session = WebSocketSession()
        activeSession.set(session)
        val webSocket = client.newWebSocket(
            request.toOkHttpRequest(),
            object : WebSocketListener() {
                override fun onOpen(webSocket: WebSocket, response: Response) {
                    if (activeSession.get() !== session) {
                        webSocket.cancel()
                        return
                    }
                    session.connectionLatency.set(
                        (response.receivedResponseAtMillis - response.sentRequestAtMillis)
                            .takeIf { it >= 0 }?.let(Duration::ofMillis),
                    )
                    session.webSocket.set(webSocket)
                    if (trySend(WebSocketEvent.Connected).isFailure) {
                        activeSession.compareAndSet(session, null)
                        webSocket.cancel()
                    }
                }

                override fun onMessage(webSocket: WebSocket, text: String) {
                    val result = trySend(WebSocketEvent.Message(text))
                    if (result.isFailure && !result.isClosed) {
                        activeSession.compareAndSet(session, null)
                        webSocket.cancel()
                        close(WebSocketOverflowException())
                    }
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
            },
        )
        awaitClose {
            activeSession.compareAndSet(session, null)
            webSocket.cancel()
        }
    }

    private fun WebSocketRequest.toOkHttpRequest(): Request = Request.Builder()
        .url(url)
        .apply {
            headers.forEach { (name, value) -> header(name, value) }
        }
        .build()

    private class WebSocketOverflowException : IllegalStateException("Message buffer overflow")

    private class WebSocketSession {
        val webSocket = AtomicReference<WebSocket?>()
        val connectionLatency = AtomicReference<Duration?>()
    }

    companion object {
        private const val TAG = "WebSocketConnection"
        private val PROBE_TIMEOUT = Duration.ofSeconds(10)
    }
}

internal fun TimeMark?.connectedDuration(): Duration = this?.elapsedNow()?.toJavaDuration() ?: Duration.ZERO
