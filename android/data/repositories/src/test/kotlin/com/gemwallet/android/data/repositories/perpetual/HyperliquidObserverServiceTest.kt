package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
import com.gemwallet.android.data.repositories.stream.WebSocketEvent
import com.gemwallet.android.model.Session
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemSubscriptionMethod

class HyperliquidObserverServiceTest {

    @Test
    fun `toWebSocketUrl converts scheme and appends ws path once`() {
        assertEquals("wss://rpc.hypercore.dev/ws", "https://rpc.hypercore.dev".toWebSocketUrl())
        assertEquals("wss://rpc.hypercore.dev/ws", "https://rpc.hypercore.dev/".toWebSocketUrl())
        assertEquals("wss://rpc.hypercore.dev/ws", "https://rpc.hypercore.dev/ws".toWebSocketUrl())
        assertEquals("ws://localhost:8545/ws", "http://localhost:8545".toWebSocketUrl())
        assertEquals("wss://api.hyperliquid.xyz/ws", "wss://api.hyperliquid.xyz".toWebSocketUrl())
    }

    @Test
    fun `subscribe sends a subscribe request over the connection`() = runTest {
        val scope = CoroutineScope(StandardTestDispatcher(testScheduler))
        val connection = RecordingConnection()
        val service = service(connection, scope)

        service.subscribe(GemPerpetualSubscription.MarketPrices)
        advanceUntilIdle()

        assertEquals(listOf(SUBSCRIBE_REQUEST), connection.sent)
        scope.cancel()
    }

    @Test
    fun `unsubscribe sends an unsubscribe request over the connection`() = runTest {
        val scope = CoroutineScope(StandardTestDispatcher(testScheduler))
        val connection = RecordingConnection()
        val service = service(connection, scope)

        service.unsubscribe(GemPerpetualSubscription.MarketPrices)
        advanceUntilIdle()

        assertEquals(listOf(UNSUBSCRIBE_REQUEST), connection.sent)
        scope.cancel()
    }

    private fun service(connection: WebSocketConnectable, scope: CoroutineScope): HyperliquidObserverService {
        val eventHandler = mockk<HyperliquidEventHandler> {
            every { chartUpdates } returns emptyFlow()
            every { subscriptionRequest(GemSubscriptionMethod.SUBSCRIBE, any()) } returns SUBSCRIBE_REQUEST
            every { subscriptionRequest(GemSubscriptionMethod.UNSUBSCRIBE, any()) } returns UNSUBSCRIBE_REQUEST
        }
        return HyperliquidObserverService(
            sessionRepository = mockk { every { session() } returns MutableStateFlow<Session?>(null) },
            userConfig = mockk { every { isPerpetualEnabled() } returns flowOf(false) },
            syncPerpetualPositions = mockk(relaxed = true),
            eventHandler = eventHandler,
            connection = connection,
            scope = scope,
        )
    }

    private class RecordingConnection : WebSocketConnectable {
        val sent = mutableListOf<String>()
        override fun connect(): Flow<WebSocketEvent> = emptyFlow()
        override suspend fun send(message: String): Boolean {
            sent.add(message)
            return true
        }
    }

    private companion object {
        const val SUBSCRIBE_REQUEST = "subscribe-request"
        const val UNSUBSCRIBE_REQUEST = "unsubscribe-request"
    }
}
