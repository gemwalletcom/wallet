package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.testkit.mockWalletConnectSessionProposal
import com.gemwallet.android.testkit.mockWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ActiveWalletConnectRequestTest {

    private val events = MutableSharedFlow<WalletConnectEvent>()

    private val verifyContext = mockWalletConnectVerifyContext()

    @Test
    fun finishWithPayloadClearsOnlyTheRequestThatProducedIt() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, backgroundScope)
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
        val activeRequest = ActiveWalletConnectRequest(events, backgroundScope)
        testScheduler.runCurrent()
        events.emit(WalletConnectEvent.SessionProposal(mockWalletConnectSessionProposal(), verifyContext))
        testScheduler.runCurrent()
        checkNotNull(activeRequest.current.value)

        events.emit(WalletConnectEvent.SessionDeleted("topic"))
        testScheduler.runCurrent()

        assertNull(activeRequest.current.value)
    }

    @Test
    fun finishWithoutPayloadClearsUnconditionally() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, backgroundScope)
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
