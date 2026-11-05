package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.testkit.mockGemWalletConnectMessageRequest
import com.gemwallet.android.testkit.mockWalletConnectSessionProposal
import com.gemwallet.android.testkit.mockWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
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

class ActiveWalletConnectRequestTest {

    private val events = MutableSharedFlow<WalletConnectEvent>()

    private val verifyContext = mockWalletConnectVerifyContext()

    @Test
    fun finishWithPayloadClearsOnlyTheRequestThatProducedIt() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, WalletConnectPendingRequests(), backgroundScope)
        testScheduler.runCurrent()
        val first = mockWalletConnectSessionProposal()
        val second = mockWalletConnectSessionRequest(id = 1)

        events.emit(WalletConnectEvent.SessionProposal(first, verifyContext))
        events.emit(WalletConnectEvent.SessionRequest(second, verifyContext))
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
        events.emit(WalletConnectEvent.SessionProposal(mockWalletConnectSessionProposal(), verifyContext))
        testScheduler.runCurrent()
        checkNotNull(activeRequest.current.value)

        events.emit(WalletConnectEvent.SessionDeleted("topic"))
        testScheduler.runCurrent()

        assertNull(activeRequest.current.value)
    }

    @Test
    fun anExpiredRequestUnderReviewIsAnsweredAsExpiredAndKeepsItsScreenUntilAnswered() = runTest {
        val pendingRequests = WalletConnectPendingRequests()
        val activeRequest = ActiveWalletConnectRequest(events, pendingRequests, backgroundScope)
        testScheduler.runCurrent()
        events.emit(WalletConnectEvent.SessionRequest(mockWalletConnectSessionRequest(id = 1), verifyContext))
        val review = backgroundScope.async { runCatching { pendingRequests.signMessage(mockGemWalletConnectMessageRequest(sessionId = "topic", requestId = "1")) } }
        testScheduler.runCurrent()

        events.emit(WalletConnectEvent.RequestExpired("topic", 1))
        testScheduler.runCurrent()

        assertTrue(review.await().exceptionOrNull() is GemServiceException.Expired)
        assertNotNull(activeRequest.current.value)
    }

    @Test
    fun anExpiredRequestStillLoadingClosesItsScreen() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, WalletConnectPendingRequests(), backgroundScope)
        testScheduler.runCurrent()
        events.emit(WalletConnectEvent.SessionRequest(mockWalletConnectSessionRequest(id = 1), verifyContext))
        testScheduler.runCurrent()

        events.emit(WalletConnectEvent.RequestExpired("topic", 1))
        testScheduler.runCurrent()

        assertNull(activeRequest.current.value)
    }

    @Test
    fun anotherRequestsExpiryLeavesTheReviewAlone() = runTest {
        val pendingRequests = WalletConnectPendingRequests()
        val activeRequest = ActiveWalletConnectRequest(events, pendingRequests, backgroundScope)
        testScheduler.runCurrent()
        events.emit(WalletConnectEvent.SessionRequest(mockWalletConnectSessionRequest(id = 1), verifyContext))
        val review = backgroundScope.async { pendingRequests.signMessage(mockGemWalletConnectMessageRequest(sessionId = "topic", requestId = "1")) }
        testScheduler.runCurrent()

        events.emit(WalletConnectEvent.RequestExpired("topic", 2))
        testScheduler.runCurrent()

        assertTrue(review.isActive)
        assertNotNull(activeRequest.current.value)
    }

    @Test
    fun finishWithoutPayloadClearsUnconditionally() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, WalletConnectPendingRequests(), backgroundScope)
        testScheduler.runCurrent()
        events.emit(WalletConnectEvent.SessionProposal(mockWalletConnectSessionProposal(), verifyContext))
        testScheduler.runCurrent()
        checkNotNull(activeRequest.current.value)

        activeRequest.finish()

        assertNull(activeRequest.current.value)
    }

    @Test
    fun everyRequestKindHasItsOwnKey() {
        val proposalKey = WalletConnectUserRequest.SessionProposal(mockWalletConnectSessionProposal(), verifyContext).key
        val firstRequest = WalletConnectUserRequest.SessionRequest(mockWalletConnectSessionRequest(id = 1), verifyContext).key
        val secondRequest = WalletConnectUserRequest.SessionRequest(mockWalletConnectSessionRequest(id = 2), verifyContext).key

        assertEquals(firstRequest, WalletConnectUserRequest.SessionRequest(mockWalletConnectSessionRequest(id = 1), verifyContext).key)
        assertEquals(3, setOf(proposalKey, firstRequest, secondRequest).size)
    }
}
