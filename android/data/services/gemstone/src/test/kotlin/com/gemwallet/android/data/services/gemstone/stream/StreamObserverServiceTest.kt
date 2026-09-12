package com.gemwallet.android.data.services.gemstone.stream

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
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
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemStreamEvent
import uniffi.gemstone.GemStreamServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class StreamObserverServiceTest {

    private val sessions = MutableStateFlow<Session?>(mockSession())
    private val getSession = mockk<GetSession> {
        every { this@mockk() } returns sessions
    }
    private val service = mockk<GemStreamServiceInterface>(relaxed = true) {
        coEvery { prepareConnection() } returns true
        coEvery { handle(any()) } returns GemStreamEvent.Prices(prices = 0u, rates = 0u)
    }
    private val connection = Connection()

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

        sessions.value = mockSession(wallet = mockWallet(id = "wallet-2"))
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
        sessions.value = mockSession(wallet = mockWallet(id = "wallet-2"))
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
        coEvery { service.handle("snapshot") } coAnswers {
            snapshotGate.await()
            GemStreamEvent.Prices(prices = 0u, rates = 0u)
        }
        observer().start()
        runCurrent()

        connection.events.emit(WebSocketEvent.Message("snapshot"))
        runCurrent()
        connection.events.emit(WebSocketEvent.Message("update"))
        runCurrent()

        coVerify(exactly = 1) { service.handle("snapshot") }
        coVerify(exactly = 0) { service.handle("update") }

        snapshotGate.complete(Unit)
        runCurrent()

        coVerifySequence {
            service.prepareConnection()
            service.connected()
            service.handle("snapshot")
            service.handle("update")
        }
    }

    @Test
    fun cancelsMessageHandlingWhenStopped() = runTest {
        val cancelled = CompletableDeferred<Unit>()
        coEvery { service.handle("snapshot") } coAnswers {
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

    private fun TestScope.observer() = StreamObserverService(
        getSession = getSession,
        service = service,
        connection = connection,
        scope = backgroundScope,
    )

    private class Connection : WebSocketConnectable {
        val events = MutableSharedFlow<WebSocketEvent>(extraBufferCapacity = 1)
        var connectCount = 0
            private set
        var activeConnections = 0
            private set

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
