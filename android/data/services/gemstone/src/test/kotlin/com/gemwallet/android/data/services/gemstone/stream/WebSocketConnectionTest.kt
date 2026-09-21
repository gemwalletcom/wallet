package com.gemwallet.android.data.services.gemstone.stream

import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import okhttp3.OkHttpClient
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemConnectionService
import java.time.Duration
import java.util.concurrent.TimeUnit

@OptIn(ExperimentalCoroutinesApi::class)
class WebSocketConnectionTest {

    @Test
    fun ignoresOpenCallbackFromCancelledConnection() = runTest {
        val firstSocket = mockk<WebSocket>(relaxed = true)
        val secondSocket = mockk<WebSocket>(relaxed = true) {
            every { send(any<String>()) } returns true
        }
        val sockets = listOf(firstSocket, secondSocket)
        val listeners = mutableListOf<WebSocketListener>()
        val client = mockk<OkHttpClient>()
        val builder = mockk<OkHttpClient.Builder>()
        every { client.newBuilder() } returns builder
        every { builder.pingInterval(any<Long>(), any<TimeUnit>()) } returns builder
        every { builder.build() } returns client
        every { client.newWebSocket(any(), capture(listeners)) } answers { sockets[listeners.lastIndex] }
        val connection = WebSocketConnection(
            requestProvider = { WebSocketRequest("wss://example.test") },
            client = client,
            connectionService = GemConnectionService(),
        )
        val response = mockk<Response> {
            every { sentRequestAtMillis } returns 1000L
            every { receivedResponseAtMillis } returns 1125L
        }
        val staleResponse = mockk<Response> {
            every { sentRequestAtMillis } returns 1000L
            every { receivedResponseAtMillis } returns 1900L
        }
        assertNull(connection.connectionLatency)
        val first = backgroundScope.launch { connection.connect().collect() }
        runCurrent()
        first.cancel()
        runCurrent()

        backgroundScope.launch { connection.connect().collect() }
        runCurrent()
        listeners[1].onOpen(secondSocket, response)
        runCurrent()
        listeners[0].onOpen(firstSocket, staleResponse)
        runCurrent()

        assertEquals(Duration.ofMillis(125), connection.connectionLatency)
        assertTrue(connection.send("subscribe"))
        verify(exactly = 1) { secondSocket.send("subscribe") }
        verify(exactly = 0) { firstSocket.send(any<String>()) }

        listeners[1].onClosed(secondSocket, 1000, "Closed")
        assertNull(connection.connectionLatency)
    }
}
