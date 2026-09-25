package com.gemwallet.android.data.services.gemstone.stream

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.connection.ConnectionComponentHealth
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ConnectionComponent
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.coVerifySequence
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemConnectionServiceInterface
import uniffi.gemstone.GemReconnection
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemStreamEvent
import uniffi.gemstone.GemStreamServiceInterface
import java.time.Duration

@OptIn(ExperimentalCoroutinesApi::class)
class StreamObserverServiceTest {

    private val sessions = MutableStateFlow<Session?>(mockSession())
    private val getSession = mockk<GetSession> {
        every { this@mockk() } returns sessions
    }
    private val service = mockk<GemStreamServiceInterface>(relaxed = true) {
        coEvery { prepareConnection() } returns true
        coEvery { decodeEvent(any()) } returns GemStreamEvent.Prices(prices = 0u, rates = 0u)
    }
    private val connection = Connection()
    private val health = ConnectionComponentHealth(ConnectionComponent.Stream)

    @Test
    fun forwardsReconnectEvents() = runTest {
        observer().start()
        runCurrent()

        connection.events.emit(WebSocketEvent.Disconnected)
        connection.events.emit(WebSocketEvent.Connected)
        runCurrent()

        coVerifySequence {
            service.prepareConnection()
            service.connected()
            service.disconnected()
            service.connected()
        }
    }

    @Test
    fun restartsAfterStopAndStart() = runTest {
        val subject = observer()
        subject.start()
        runCurrent()

        subject.stop()
        subject.start()
        runCurrent()

        assertEquals(2, connection.connectCount)
        coVerifySequence {
            service.prepareConnection()
            service.connected()
            service.disconnected()
            service.prepareConnection()
            service.connected()
        }
    }

    @Test
    fun aWalletChangeSubscribesOverTheLiveConnection() = runTest {
        observer().start()
        runCurrent()

        sessions.value = mockSession(wallet = mockWallet(id = WalletId("wallet-2")))
        runCurrent()

        assertEquals(1, connection.connectCount)
        assertEquals(1, connection.activeConnections)
        coVerify(exactly = 1) { service.prepareConnection() }
        coVerify(exactly = 1) { service.updateSession() }
    }

    @Test
    fun currencyChangeDoesNotCancelPreparation() = runTest {
        val preparationGate = CompletableDeferred<Unit>()
        coEvery { service.prepareConnection() } coAnswers {
            preparationGate.await()
            true
        }
        observer().start()
        runCurrent()

        sessions.value = mockSession(currency = Currency.EUR)
        runCurrent()
        preparationGate.complete(Unit)
        runCurrent()

        coVerify(exactly = 1) { service.prepareConnection() }
        assertEquals(1, connection.connectCount)
    }

    @Test
    fun startsWhenCoreAllowsConnection() = runTest {
        sessions.value = null
        coEvery { service.prepareConnection() } returns false
        observer().start()
        runCurrent()

        assertEquals(0, connection.connectCount)

        coEvery { service.prepareConnection() } returns true
        sessions.value = mockSession()
        runCurrent()

        assertEquals(1, connection.connectCount)
        coVerify(exactly = 2) { service.prepareConnection() }
    }

    @Test
    fun usesCoreEligibilityForAnEmptySession() = runTest {
        sessions.value = null
        observer().start()
        runCurrent()

        assertEquals(1, connection.connectCount)
        coVerify(exactly = 1) { service.prepareConnection() }
    }

    @Test
    fun staysStoppedOnSessionChanges() = runTest {
        val subject = observer()
        subject.start()
        runCurrent()

        subject.stop()
        runCurrent()
        sessions.value = mockSession(wallet = mockWallet(id = WalletId("wallet-2")))
        runCurrent()

        assertEquals(1, connection.connectCount)
        assertEquals(0, connection.activeConnections)
        coVerify(exactly = 1) { service.prepareConnection() }
        coVerify(exactly = 1) { service.disconnected() }
    }

    @Test
    fun startsOnlyOneConnection() = runTest {
        val subject = observer()
        subject.start()
        subject.start()
        runCurrent()

        assertEquals(1, connection.connectCount)
    }

    @Test
    fun cancelsPreparationWhenStopped() = runTest {
        val cancelled = CompletableDeferred<Unit>()
        coEvery { service.prepareConnection() } coAnswers {
            try {
                awaitCancellation()
            } finally {
                cancelled.complete(Unit)
            }
        }
        val subject = observer()
        subject.start()
        runCurrent()

        subject.stop()
        runCurrent()

        assertTrue(cancelled.isCompleted)
        assertEquals(0, connection.connectCount)
    }

    @Test
    fun waitsForDisconnectedAcrossRapidRestarts() = runTest {
        val cleanupGate = CompletableDeferred<Unit>()
        coEvery { service.disconnected() } coAnswers { cleanupGate.await() }
        val subject = observer()
        subject.start()
        runCurrent()

        try {
            subject.stop()
            subject.start()
            runCurrent()
            subject.stop()
            subject.start()
            runCurrent()

            assertEquals(1, connection.connectCount)
        } finally {
            cleanupGate.complete(Unit)
        }
        runCurrent()

        assertEquals(2, connection.connectCount)
        assertEquals(1, connection.activeConnections)
    }

    @Test
    fun doesNotConnectAfterCancelledPreparationCompletes() = runTest {
        val preparationGate = CompletableDeferred<Unit>()
        coEvery { service.prepareConnection() } coAnswers {
            withContext(NonCancellable) { preparationGate.await() }
            true
        }
        val subject = observer()
        subject.start()
        runCurrent()

        subject.stop()
        preparationGate.complete(Unit)
        runCurrent()

        assertEquals(0, connection.connectCount)
    }

    @Test
    fun handlesMessagesInOrder() = runTest {
        val snapshotGate = CompletableDeferred<Unit>()
        coEvery { service.decodeEvent("snapshot") } coAnswers {
            snapshotGate.await()
            GemStreamEvent.Prices(prices = 0u, rates = 0u)
        }
        observer().start()
        runCurrent()

        connection.events.emit(WebSocketEvent.Message("snapshot"))
        runCurrent()
        connection.events.emit(WebSocketEvent.Message("update"))
        runCurrent()

        coVerify(exactly = 1) { service.decodeEvent("snapshot") }
        coVerify(exactly = 0) { service.decodeEvent("update") }

        snapshotGate.complete(Unit)
        runCurrent()

        coVerifyOrder {
            service.prepareConnection()
            service.connected()
            service.decodeEvent("snapshot")
            service.decodeEvent("update")
        }
    }

    @Test
    fun aSlowSyncDoesNotHoldBackTheNextMessage() = runTest {
        val balances = GemStreamEvent.Balances(walletId = "multicoin_0x1", assetIds = emptyList())
        val syncGate = CompletableDeferred<Unit>()
        coEvery { service.decodeEvent("balances") } returns balances
        coEvery { service.sync(balances) } coAnswers { syncGate.await() }
        observer().start()
        runCurrent()

        connection.events.emit(WebSocketEvent.Message("balances"))
        runCurrent()
        connection.events.emit(WebSocketEvent.Message("prices"))
        runCurrent()

        coVerify(exactly = 1) { service.sync(balances) }
        coVerify(exactly = 1) { service.decodeEvent("prices") }
        syncGate.complete(Unit)
    }

    @Test
    fun cancelsMessageHandlingWhenStopped() = runTest {
        val cancelled = CompletableDeferred<Unit>()
        coEvery { service.decodeEvent("snapshot") } coAnswers {
            try {
                awaitCancellation()
            } finally {
                cancelled.complete(Unit)
            }
        }
        val subject = observer()
        subject.start()
        runCurrent()

        connection.events.emit(WebSocketEvent.Message("snapshot"))
        runCurrent()
        subject.stop()
        runCurrent()

        assertTrue(cancelled.isCompleted)
    }

    private val connectionService = mockk<GemConnectionServiceInterface> {
        every { reconnection(any(), any()) } returns GemReconnection(nextAttempt = 1u, delay = Duration.ofSeconds(1))
    }

    @Test
    fun aFailedPreparationWaitsBeforeTryingAgain() = runTest {
        coEvery { service.prepareConnection() } throws GemServiceException.Store("disk")
        val subject = observer()
        subject.start()
        runCurrent()
        coVerify(exactly = 1) { service.prepareConnection() }

        advanceTimeBy(999)
        runCurrent()
        coVerify(exactly = 1) { service.prepareConnection() }

        advanceTimeBy(1)
        runCurrent()
        coVerify(exactly = 2) { service.prepareConnection() }

        subject.stop()
        advanceTimeBy(60_000)
        runCurrent()
        coVerify(exactly = 2) { service.prepareConnection() }
    }

    @Test
    fun aFailedFirstSubscriptionIsNotHealthyAndRecoversByReconnecting() = runTest {
        var subscriptions = 0
        coEvery { service.connected() } answers {
            subscriptions++
            if (subscriptions == 1) throw GemServiceException.Api("subscribe")
        }
        val reports = mutableListOf<Boolean>()
        backgroundScope.launch { health.healthFlow().collect { reports.add(it) } }

        observer().start()
        runCurrent()
        assertEquals(listOf(false), reports)
        assertEquals(1, connection.connectCount)

        advanceTimeBy(1_000)
        runCurrent()
        assertEquals(2, connection.connectCount)
        assertEquals(listOf(false, true), reports)
    }

    private fun TestScope.observer() = StreamObserverService(
        getSession = getSession,
        service = service,
        connection = connection,
        health = health,
        connectionService = connectionService,
        scope = backgroundScope,
    )

    private class Connection : WebSocketConnectable {
        val events = MutableSharedFlow<WebSocketEvent>(extraBufferCapacity = 1)
        var connectCount = 0
            private set
        var activeConnections = 0
            private set

        override val connectionLatency: Duration? = null

        override val isConnected: Boolean
            get() = activeConnections > 0

        override fun connect() = flow {
            connectCount++
            activeConnections++
            try {
                emit(WebSocketEvent.Connected)
                emitAll(events)
            } finally {
                activeConnections--
            }
        }

        override suspend fun send(message: String) = isConnected
    }
}
