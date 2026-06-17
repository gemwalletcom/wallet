package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
import com.gemwallet.android.data.repositories.stream.WebSocketEvent
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.asFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

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
    fun `on connect resubscribes defaults and routes messages to the event handler`() = runTest {
        val scope = CoroutineScope(StandardTestDispatcher(testScheduler))
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.HyperCore, address = ADDRESS)))
        val connection = RecordingConnection(listOf(WebSocketEvent.Connected, WebSocketEvent.Message(MESSAGE)))
        val eventHandler = mockk<HyperliquidEventHandler>(relaxed = true) {
            every { chartUpdates } returns emptyFlow()
        }
        val observePerpetualWallet = mockk<ObservePerpetualWallet>()
        every { observePerpetualWallet() } returns flowOf(wallet)
        val observer = HyperliquidObserverService(
            observePerpetualWallet = observePerpetualWallet,
            syncPerpetualPositions = mockk(relaxed = true),
            eventHandler = eventHandler,
            subscriptionService = HyperliquidSubscriptionService { _, _ -> SUBSCRIBE_REQUEST },
            connection = connection,
            scope = scope,
        )

        observer.start()
        advanceUntilIdle()

        assertEquals(listOf(SUBSCRIBE_REQUEST, SUBSCRIBE_REQUEST), connection.sent)
        coVerify { eventHandler.handle(wallet.id, MESSAGE) }
        scope.cancel()
    }

    private class RecordingConnection(private val events: List<WebSocketEvent>) : WebSocketConnectable {
        val sent = mutableListOf<String>()
        override fun connect(): Flow<WebSocketEvent> = events.asFlow()
        override suspend fun send(message: String): Boolean {
            sent.add(message)
            return true
        }
    }

    private companion object {
        const val ADDRESS = "0xabc"
        const val MESSAGE = "message"
        const val SUBSCRIBE_REQUEST = "subscribe-request"
    }
}
