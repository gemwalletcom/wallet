package com.gemwallet.android.application.wallet_connect

import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ActiveWalletConnectRequestTest {

    private val events = MutableSharedFlow<WalletConnectEvent>()

    private fun proposal(name: String) = WalletConnectSessionProposal(
        name = name,
        description = "",
        url = "https://example.com",
        icons = emptyList(),
        requiredNamespaces = emptyMap(),
        optionalNamespaces = emptyMap(),
        proposerPublicKey = "key-$name",
        properties = null,
    )

    private fun sessionRequest(id: Long) = WalletConnectSessionRequest(
        topic = "topic",
        chainId = "eip155:1",
        request = WalletConnectJsonRpcRequest(id = id, method = "personal_sign", params = ""),
    )

    private val verifyContext = WalletConnectVerifyContext(
        origin = "https://example.com",
        validation = WalletConnectValidation.Valid,
        isScam = false,
    )

    @Test
    fun finishWithPayloadClearsOnlyTheRequestThatProducedIt() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, backgroundScope)
        testScheduler.runCurrent()
        val first = proposal("first")
        val second = sessionRequest(1)

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
    fun finishWithoutPayloadClearsUnconditionally() = runTest {
        val activeRequest = ActiveWalletConnectRequest(events, backgroundScope)
        testScheduler.runCurrent()
        events.emit(WalletConnectEvent.SessionProposal(proposal("only"), verifyContext))
        testScheduler.runCurrent()
        checkNotNull(activeRequest.current.value)

        activeRequest.finish()

        assertNull(activeRequest.current.value)
    }
}
