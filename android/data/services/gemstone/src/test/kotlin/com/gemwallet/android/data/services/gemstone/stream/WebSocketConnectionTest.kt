package com.gemwallet.android.data.services.gemstone.stream

import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import okhttp3.OkHttpClient
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemConnectionService
import java.time.Duration

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
        every { builder.pingInterval(any<java.time.Duration>()) } returns builder
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

    @Test
    fun anOverflowWhileConsumptionIsSuspendedReconnects() = runTest {
        val sockets = listOf(mockk<WebSocket>(relaxed = true), mockk<WebSocket>(relaxed = true))
        val listeners = mutableListOf<WebSocketListener>()
        val client = mockk<OkHttpClient>()
        val builder = mockk<OkHttpClient.Builder>()
        every { client.newBuilder() } returns builder
        every { builder.pingInterval(any<java.time.Duration>()) } returns builder
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
        val gate = CompletableDeferred<Unit>()
        val events = mutableListOf<WebSocketEvent>()
        backgroundScope.launch {
            connection.connect().collect { event ->
                events.add(event)
                if (event is WebSocketEvent.Message) gate.await()
            }
        }
        runCurrent()
        listeners[0].onOpen(sockets[0], response)
        runCurrent()

        repeat(500) { listeners[0].onMessage(sockets[0], "message $it") }
        runCurrent()

        verify { sockets[0].cancel() }
        assertFalse(connection.isConnected)

        gate.complete(Unit)
        advanceTimeBy(60_000)
        runCurrent()

        assertTrue(events.contains(WebSocketEvent.Disconnected))
        assertEquals(2, listeners.size)
    }
}
