package com.gemwallet.android.application.wallet_connect

import android.util.Log
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletConnectResponse
import uniffi.gemstone.GemWalletConnectRpcError
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectResponseType

class WalletConnectRequestHandlerTest {
    private val request = WalletConnectSessionRequest(
        topic = "topic",
        chainId = "eip155:1",
        request = WalletConnectJsonRpcRequest(id = 1, method = "personal_sign", params = "[]"),
    )

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.e(any(), any(), any()) } returns 0
    }

    @After
    fun tearDown() = unmockkAll()

    @Test
    fun `core response is returned as json rpc result`() = runTest {
        val service = mockk<GemWalletConnectServiceInterface> {
            coEvery { handleRequest(any()) } returns GemWalletConnectResponse.Response(WalletConnectResponseType.String("0xsig"))
        }

        assertEquals(WalletConnectJsonRpcResponse.Result("0xsig"), WalletConnectRequestHandler(service).handle(request, "https://dapp"))
    }

    @Test
    fun `unsupported methods and failures map to json rpc errors`() = runTest {
        val notFound = mockk<GemWalletConnectServiceInterface> {
            coEvery { handleRequest(any()) } returns GemWalletConnectResponse.MethodNotFound
        }
        val failing = mockk<GemWalletConnectServiceInterface> {
            coEvery { handleRequest(any()) } throws IllegalStateException("rejected")
            every { userRejectedError() } returns GemWalletConnectRpcError(code = 4001, message = "User rejected the request")
        }

        assertEquals(WalletConnectJsonRpcResponse.Error(-32601, "Method not found"), WalletConnectRequestHandler(notFound).handle(request, "https://dapp"))
        val failure = runCatching { WalletConnectRequestHandler(failing).handle(request, "https://dapp") }.exceptionOrNull()
        assertTrue(failure is IllegalStateException)
        assertEquals(WalletConnectJsonRpcResponse.Error(4001, "User rejected the request"), WalletConnectRequestHandler(failing).handle(request.copy(chainId = null), "https://dapp"))
    }
}
