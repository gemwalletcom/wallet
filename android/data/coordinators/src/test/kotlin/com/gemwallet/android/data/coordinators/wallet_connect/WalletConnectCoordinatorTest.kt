package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.data.services.gemstone.stream.WebSocketEvent
import com.gemwallet.android.testkit.mockGemWalletConnectProposal
import com.gemwallet.android.testkit.mockGemWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletConnectFailure

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectCoordinatorTest {

    private val socket = WebSocketConnectionMock()
    private val request = mockGemWalletConnectSessionRequest(topic = "topic-1", requestId = "42")

    private fun TestScope.coordinator(client: WalletConnectClientMock, storedSessions: Boolean = false) = WalletConnectCoordinator(
        client = client,
        relay = WalletConnectRelay(socket),
        walletConnectService = mockk { coEvery { hasSessions() } returns storedSessions },
        scope = backgroundScope,
    )

    @Test
    fun `a launch with sessions restores them and leaves the relay to the foreground`() = runTest {
        val client = WalletConnectClientMock()

        coordinator(client, storedSessions = true)
        runCurrent()

        assertEquals(listOf("restore"), client.calls)
        assertFalse("a process started in the background keeps no relay open", socket.isCollected)
    }

    @Test
    fun `a launch without sessions leaves the relay closed`() = runTest {
        val client = WalletConnectClientMock()

        coordinator(client, storedSessions = false)
        runCurrent()

        assertEquals(emptyList<String>(), client.calls)
        assertFalse(socket.isCollected)
    }

    @Test
    fun `the foreground opens one relay connection while the client has topics`() = runTest {
        val subject = coordinator(WalletConnectClientMock(needsConnection = true))

        subject.start()
        subject.start()
        runCurrent()

        assertTrue(socket.isCollected)
        assertEquals(1, socket.connections)
    }

    @Test
    fun `the foreground leaves the relay closed without topics`() = runTest {
        val subject = coordinator(WalletConnectClientMock(needsConnection = false))

        subject.start()
        runCurrent()

        assertFalse(socket.isCollected)
    }

    @Test
    fun `the background closes the relay and the next foreground opens it again`() = runTest {
        val subject = coordinator(WalletConnectClientMock(needsConnection = true))
        subject.start()
        runCurrent()

        subject.stop()
        runCurrent()
        assertFalse(socket.isCollected)

        subject.start()
        runCurrent()
        assertTrue(socket.isCollected)
        assertEquals(2, socket.connections)
    }

    @Test
    fun `relay frames reach the client in order and a failing frame leaves the next one handled`() = runTest {
        val client = WalletConnectClientMock(needsConnection = true, receiving = { if (it == "broken") throw GemServiceException.InvalidInput("broken") })
        coordinator(client).start()
        runCurrent()

        socket.events.emit(WebSocketEvent.Connected)
        socket.events.emit(WebSocketEvent.Message("broken"))
        socket.events.emit(WebSocketEvent.Message("frame"))
        runCurrent()

        assertEquals(listOf("connected", "receive broken", "receive frame"), client.calls)
    }

    @Test
    fun `pairing opens the relay and pairs through Core`() = runTest {
        val client = WalletConnectClientMock()
        var paired = false

        coordinator(client).pair("wc:uri", onSuccess = { paired = true }, onError = {})
        runCurrent()

        assertTrue(paired)
        assertTrue(socket.isCollected)
        assertEquals(listOf("pair wc:uri"), client.calls)
    }

    @Test
    fun `a pairing Core refuses reports its error`() = runTest {
        val errors = mutableListOf<GemErrorText>()

        coordinator(WalletConnectClientMock(pairing = { throw GemServiceException.Expired() })).pair("wc:uri", onSuccess = {}, onError = errors::add)
        runCurrent()

        assertEquals(1, errors.size)
    }

    @Test
    fun `disconnecting opens the relay and disconnects through Core`() = runTest {
        val client = WalletConnectClientMock()
        var disconnected = false

        coordinator(client).disconnect("topic-1", onSuccess = { disconnected = true }, onError = {})
        runCurrent()

        assertTrue(disconnected)
        assertTrue(socket.isCollected)
        assertEquals(listOf("disconnect topic-1"), client.calls)
    }

    @Test
    fun `an approval goes to Core with the wallet that approved it`() = runTest {
        val client = WalletConnectClientMock()
        var approved = false

        coordinator(client).approveConnection(mockWallet(id = WalletId("wallet-1")), mockGemWalletConnectProposal(id = "key"), onSuccess = { approved = true }, onError = {})
        runCurrent()

        assertTrue(approved)
        assertEquals(listOf("approveSession key wallet-1"), client.calls)
    }

    @Test
    fun `wallets the user switches to reach Core one at a time in the order chosen`() = runTest {
        val firstFollowed = CompletableDeferred<Unit>()
        val client = WalletConnectClientMock(following = { if (it.id == "first") firstFollowed.await() })
        val subject = coordinator(client)

        subject.followWallet(mockWallet(id = WalletId("first")))
        subject.followWallet(mockWallet(id = WalletId("second")))
        runCurrent()
        assertEquals(listOf("followWallet first"), client.calls)

        firstFollowed.complete(Unit)
        runCurrent()

        assertEquals(listOf("followWallet first", "followWallet second"), client.calls)
    }

    @Test
    fun `a request Core answers with a failure reports it and finishes`() = runTest {
        val failures = mutableListOf<GemWalletConnectFailure>()
        var finished = false

        coordinator(WalletConnectClientMock(answering = { _, _ -> GemWalletConnectFailure.MaliciousOrigin })).respondRequest(request, onFailure = failures::add) { finished = true }
        runCurrent()

        assertEquals(listOf(GemWalletConnectFailure.MaliciousOrigin), failures)
        assertTrue(finished)
    }

    @Test
    fun `a request Core fails to answer still finishes`() = runTest {
        val failures = mutableListOf<GemWalletConnectFailure>()
        var finished = false

        coordinator(WalletConnectClientMock(answering = { _, _ -> throw GemServiceException.Offline() })).respondRequest(request, onFailure = failures::add) { finished = true }
        runCurrent()

        assertEquals(emptyList<GemWalletConnectFailure>(), failures)
        assertTrue(finished)
    }

    @Test
    fun `rejecting a request Core is still answering cancels the answer and refuses it once`() = runTest {
        var answerCancelled = false
        val client = WalletConnectClientMock(
            answering = { _, _ ->
                try {
                    awaitCancellation()
                } finally {
                    answerCancelled = true
                }
            },
        )
        val subject = coordinator(client)
        var answerFinished = false
        var rejections = 0

        subject.respondRequest(request, onFailure = {}) { answerFinished = true }
        runCurrent()
        subject.rejectRequest(request) { rejections += 1 }
        subject.rejectRequest(request) { rejections += 1 }
        runCurrent()

        assertEquals(listOf("answerRequest topic-1 42", "rejectRequest topic-1 42"), client.calls)
        assertTrue(answerCancelled)
        assertFalse(answerFinished)
        assertEquals(1, rejections)
    }

    @Test
    fun `rejecting an answered request leaves it alone`() = runTest {
        val client = WalletConnectClientMock()
        val subject = coordinator(client)

        subject.respondRequest(request, onFailure = {}, onFinish = {})
        runCurrent()
        subject.rejectRequest(request) {}
        runCurrent()

        assertEquals(listOf("answerRequest topic-1 42"), client.calls)
    }
}
