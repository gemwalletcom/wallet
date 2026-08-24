package com.gemwallet.android.data.repositories.perpetual

import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
import com.gemwallet.android.data.repositories.stream.WebSocketEvent
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualAccountMode
import io.mockk.coEvery
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
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.HyperliquidSubscriptions

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
            syncPerpetuals = mockk(relaxed = true),
            syncPerpetualPositions = mockk(relaxed = true),
            getPerpetualAccountMode = mockk { coEvery { getPerpetualAccountMode(any(), any()) } returns PerpetualAccountMode.Standard },
            eventHandler = eventHandler,
            subscriptionService = HyperliquidSubscriptionService(HyperliquidSubscriptions()),
            connection = connection,
            scope = scope,
        )

        observer.start()
        advanceUntilIdle()

        assertEquals(
            setOf(
                """{"method":"subscribe","subscription":{"type":"clearinghouseState","user":"0xabc"}}""",
                """{"method":"subscribe","subscription":{"type":"openOrders","user":"0xabc"}}""",
            ),
            connection.sent.toSet(),
        )
        coVerify { eventHandler.handle(wallet.id, PerpetualAccountMode.Standard, MESSAGE) }
        scope.cancel()
    }


    @Test
    fun `subscription requested before connect is sent only once on connect`() = runTest {
        val scope = CoroutineScope(StandardTestDispatcher(testScheduler))
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = Chain.HyperCore, address = ADDRESS)))
        val connection = RecordingConnection(listOf(WebSocketEvent.Connected))
        val eventHandler = mockk<HyperliquidEventHandler>(relaxed = true) {
            every { chartUpdates } returns emptyFlow()
        }
        val observePerpetualWallet = mockk<ObservePerpetualWallet>()
        every { observePerpetualWallet() } returns flowOf(wallet)
        val observer = HyperliquidObserverService(
            observePerpetualWallet = observePerpetualWallet,
            syncPerpetuals = mockk(relaxed = true),
            syncPerpetualPositions = mockk(relaxed = true),
            getPerpetualAccountMode = mockk { coEvery { getPerpetualAccountMode(any(), any()) } returns PerpetualAccountMode.Standard },
            eventHandler = eventHandler,
            subscriptionService = HyperliquidSubscriptionService(HyperliquidSubscriptions()),
            connection = connection,
            scope = scope,
        )

        observer.subscribe(GemPerpetualSubscription.MarketData("UNI"))
        observer.start()
        advanceUntilIdle()

        assertEquals(
            setOf(
                """{"method":"subscribe","subscription":{"type":"clearinghouseState","user":"0xabc"}}""",
                """{"method":"subscribe","subscription":{"type":"openOrders","user":"0xabc"}}""",
                """{"method":"subscribe","subscription":{"type":"activeAssetCtx","coin":"UNI"}}""",
            ),
            connection.sent.toSet(),
        )
        assertEquals(3, connection.sent.size)
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
    }
}
