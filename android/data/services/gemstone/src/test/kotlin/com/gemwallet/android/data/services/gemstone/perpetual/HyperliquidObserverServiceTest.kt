package com.gemwallet.android.data.services.gemstone.perpetual

import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnectable
import com.gemwallet.android.data.services.gemstone.stream.WebSocketEvent
import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualStreamServiceInterface
import uniffi.gemstone.GemPerpetualSubscription
import uniffi.gemstone.GemServiceException
import java.time.Duration
import java.util.concurrent.atomic.AtomicInteger

@OptIn(ExperimentalCoroutinesApi::class)
class HyperliquidObserverServiceTest {

    private val observePerpetualWallet = mockk<ObservePerpetualWallet> {
        every { this@mockk() } returns flowOf(mockWallet())
    }
    private val perpetualService = mockk<GemPerpetualServiceInterface>()
    private val streamService = mockk<GemPerpetualStreamServiceInterface>(relaxed = true)
    private val connection = Connection()

    @Test
    fun opensTheSocketWhileTheAccountModeLoads() = runTest {
        val address = CompletableDeferred<String?>()
        coEvery { perpetualService.connectionAddress(any()) } coAnswers { address.await() }
        observer().start()
        runCurrent()

        assertEquals(1, connection.activeConnections.get())
        coVerify(exactly = 0) { streamService.connected(any()) }

        address.complete("0x1")
        runCurrent()

        coVerify(exactly = 1) { streamService.connected("0x1") }
    }

    @Test
    fun closesTheSocketWhenTheAccountCannotBeLoaded() = runTest {
        coEvery { perpetualService.connectionAddress(any()) } throws GemServiceException.Store("disk")
        observer().start()
        runCurrent()

        assertEquals(1, connection.connectCount.get())
        assertEquals(0, connection.activeConnections.get())
        coVerify(exactly = 0) { streamService.connected(any()) }
    }

    @Test
    fun checksEnablementAgainWhenTheAppReturnsToTheForeground() = runTest {
        every { observePerpetualWallet() } returnsMany listOf(flowOf(null), flowOf(mockWallet()))
        coEvery { perpetualService.connectionAddress(any()) } returns "0x1"
        val subject = observer()
        subject.start()
        runCurrent()

        assertEquals(0, connection.connectCount.get())

        subject.stop()
        runCurrent()
        subject.start()
        runCurrent()

        assertEquals(1, connection.activeConnections.get())
    }

    @Test
    fun tellsCoreTheConnectionEndedWhenTheAppLeavesTheForeground() = runTest {
        coEvery { perpetualService.connectionAddress(any()) } returns "0x1"
        val subject = observer()
        subject.start()
        runCurrent()
        coVerify(exactly = 1) { streamService.connected("0x1") }

        subject.stop()
        runCurrent()

        assertEquals(0, connection.activeConnections.get())
        coVerify(exactly = 1) { streamService.disconnected() }
    }

    @Test
    fun hasCoreSubscribeAndUnsubscribeInTheOrderTheScreensAsk() = runTest {
        val market = GemPerpetualSubscription.MarketData("BTC")
        val subject = observer()

        subject.unsubscribe(market)
        subject.subscribe(market)

        coVerifyOrder {
            streamService.unsubscribe(market)
            streamService.subscribe(market)
        }
    }

    private fun TestScope.observer() = HyperliquidObserverService(
        observePerpetualWallet = observePerpetualWallet,
        perpetualService = perpetualService,
        streamService = streamService,
        connection = connection,
        scope = backgroundScope,
    )

    private class Connection : WebSocketConnectable {
        val connectCount = AtomicInteger()
        val activeConnections = AtomicInteger()

        override val connectionLatency: Duration? = null

        override val isConnected: Boolean
            get() = activeConnections.get() > 0

        override fun connect() = flow {
            connectCount.incrementAndGet()
            activeConnections.incrementAndGet()
            try {
                emit(WebSocketEvent.Connected)
                awaitCancellation()
            } finally {
                activeConnections.decrementAndGet()
            }
        }

        override suspend fun send(message: String) = isConnected
    }
}
