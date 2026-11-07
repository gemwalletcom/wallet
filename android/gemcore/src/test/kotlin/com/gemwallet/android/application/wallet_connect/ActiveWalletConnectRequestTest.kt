package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.testkit.mockGemWalletConnectAuthentication
import com.gemwallet.android.testkit.mockGemWalletConnectMessageRequest
import com.gemwallet.android.testkit.mockGemWalletConnectProposal
import com.gemwallet.android.testkit.mockGemWalletConnectSessionRequest
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletConnectEvent

class ActiveWalletConnectRequestTest {

    private val events = MutableSharedFlow<GemWalletConnectEvent>()

    @Test
    fun finishWithPayloadClearsOnlyTheRequestThatProducedIt() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, WalletConnectPendingRequests(), backgroundScope)
        testScheduler.runCurrent()
        val first = mockGemWalletConnectProposal(id = "key")
        val second = mockGemWalletConnectSessionRequest(topic = "topic", requestId = "1")

        events.emit(GemWalletConnectEvent.Proposal(first))
        events.emit(GemWalletConnectEvent.Request(second))
        testScheduler.runCurrent()

        assertFalse(activeRequest.finish(first))
        assertEquals(second, (activeRequest.current.value as WalletConnectUserRequest.SessionRequest).request)

        assertTrue(activeRequest.finish(second))
        assertNull(activeRequest.current.value)
        assertFalse(activeRequest.finish(second))
    }

    @Test
    fun aDeletedSessionClearsThePendingRequest() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, WalletConnectPendingRequests(), backgroundScope)
        testScheduler.runCurrent()
        events.emit(GemWalletConnectEvent.Proposal(mockGemWalletConnectProposal()))
        testScheduler.runCurrent()
        checkNotNull(activeRequest.current.value)

        events.emit(GemWalletConnectEvent.SessionDeleted("topic"))
        testScheduler.runCurrent()

        assertNull(activeRequest.current.value)
    }

    @Test
    fun anExpiredRequestUnderReviewIsAnsweredAsExpiredAndKeepsItsScreenUntilAnswered() = runTest {
        val pendingRequests = WalletConnectPendingRequests()
        val activeRequest = ActiveWalletConnectRequest(events, pendingRequests, backgroundScope)
        testScheduler.runCurrent()
        events.emit(GemWalletConnectEvent.Request(mockGemWalletConnectSessionRequest(topic = "topic", requestId = "1")))
        val review = backgroundScope.async { runCatching { pendingRequests.signMessage(mockGemWalletConnectMessageRequest(sessionId = "topic", requestId = "1")) } }
        testScheduler.runCurrent()

        events.emit(GemWalletConnectEvent.RequestExpired("topic", "1"))
        testScheduler.runCurrent()

        assertTrue(review.await().exceptionOrNull() is GemServiceException.Expired)
        assertNotNull(activeRequest.current.value)
    }

    @Test
    fun anExpiredRequestStillLoadingClosesItsScreen() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, WalletConnectPendingRequests(), backgroundScope)
        testScheduler.runCurrent()
        events.emit(GemWalletConnectEvent.Request(mockGemWalletConnectSessionRequest(topic = "topic", requestId = "1")))
        testScheduler.runCurrent()

        events.emit(GemWalletConnectEvent.RequestExpired("topic", "1"))
        testScheduler.runCurrent()

        assertNull(activeRequest.current.value)
    }

    @Test
    fun anotherRequestsExpiryLeavesTheReviewAlone() = runTest {
        val pendingRequests = WalletConnectPendingRequests()
        val activeRequest = ActiveWalletConnectRequest(events, pendingRequests, backgroundScope)
        testScheduler.runCurrent()
        events.emit(GemWalletConnectEvent.Request(mockGemWalletConnectSessionRequest(topic = "topic", requestId = "1")))
        val review = backgroundScope.async { pendingRequests.signMessage(mockGemWalletConnectMessageRequest(sessionId = "topic", requestId = "1")) }
        testScheduler.runCurrent()

        events.emit(GemWalletConnectEvent.RequestExpired("topic", "2"))
        testScheduler.runCurrent()

        assertTrue(review.isActive)
        assertNotNull(activeRequest.current.value)
    }

    @Test
    fun finishWithoutPayloadClearsUnconditionally() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, WalletConnectPendingRequests(), backgroundScope)
        testScheduler.runCurrent()
        events.emit(GemWalletConnectEvent.Authentication(mockGemWalletConnectAuthentication()))
        testScheduler.runCurrent()
        checkNotNull(activeRequest.current.value)

        activeRequest.finish()

        assertNull(activeRequest.current.value)
    }

    @Test
    fun everyRequestKindHasItsOwnKey() {
        val proposalKey = WalletConnectUserRequest.SessionProposal(mockGemWalletConnectProposal(id = "1")).key
        val authenticationKey = WalletConnectUserRequest.AuthenticationRequest(mockGemWalletConnectAuthentication(id = "1")).key
        val firstRequest = WalletConnectUserRequest.SessionRequest(mockGemWalletConnectSessionRequest(topic = "topic", requestId = "1")).key
        val secondRequest = WalletConnectUserRequest.SessionRequest(mockGemWalletConnectSessionRequest(topic = "topic", requestId = "2")).key

        assertEquals(firstRequest, WalletConnectUserRequest.SessionRequest(mockGemWalletConnectSessionRequest(topic = "topic", requestId = "1")).key)
        assertEquals(4, setOf(proposalKey, authenticationKey, firstRequest, secondRequest).size)
    }
}
