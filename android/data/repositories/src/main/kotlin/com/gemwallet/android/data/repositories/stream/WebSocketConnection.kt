package com.gemwallet.android.data.repositories.stream

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.client.request.header
import io.ktor.websocket.Frame
import io.ktor.websocket.readText
import io.ktor.websocket.send
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ProducerScope
import kotlinx.coroutines.channels.SendChannel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext

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
    fun connect(): Flow<WebSocketEvent>
    suspend fun send(message: String): Boolean
}

class WebSocketConnection(
    private val requestProvider: suspend () -> WebSocketRequest,
    private val reconnection: ExponentialReconnection = ExponentialReconnection(),
    private val pingInterval: Long = PING_INTERVAL_MS,
) : WebSocketConnectable {
    private val client by lazy(LazyThreadSafetyMode.SYNCHRONIZED) { createClient() }

    private val mutex = Mutex()
    private var sendChannel: SendChannel<String>? = null

    override fun connect(): Flow<WebSocketEvent> = channelFlow {
        var reconnectAttempt = 0
        while (isActive) {
            runCatchingCancellable {
                val request = requestProvider()
                client.webSocket(
                    urlString = request.url,
                    request = { request.headers.forEach { (key, value) -> header(key, value) } },
                ) {
                    reconnectAttempt = 0
                    observeSession(this@channelFlow)
                }
            }.onFailure { Log.e(TAG, "Connection error", it) }
            send(WebSocketEvent.Disconnected)
            delay(reconnection.reconnectAfterMs(reconnectAttempt))
            reconnectAttempt++
        }
    }

    override suspend fun send(message: String): Boolean = mutex.withLock {
        sendChannel?.trySend(message)?.isSuccess == true
    }

    private suspend fun DefaultClientWebSocketSession.observeSession(events: ProducerScope<WebSocketEvent>) {
        val messages = Channel<String>(Channel.UNLIMITED)
        try {
            mutex.withLock { sendChannel = messages }
            launch {
                for (message in messages) {
                    runCatchingCancellable { send(message) }
                        .onFailure { Log.e(TAG, "Send message error", it) }
                }
            }
            events.send(WebSocketEvent.Connected)
            for (frame in incoming) {
                if (frame is Frame.Text) {
                    events.send(WebSocketEvent.Message(frame.readText()))
                }
            }
        } finally {
            withContext(NonCancellable) {
                mutex.withLock {
                    if (sendChannel === messages) sendChannel = null
                }
                messages.close()
            }
        }
    }

    private fun createClient(): HttpClient = HttpClient(CIO) {
        install(WebSockets) {
            pingIntervalMillis = pingInterval
        }
    }

    companion object {
        private const val TAG = "WebSocketConnection"
        private const val PING_INTERVAL_MS = 30_000L
    }
}
