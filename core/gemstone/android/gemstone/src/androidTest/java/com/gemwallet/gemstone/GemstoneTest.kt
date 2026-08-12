package com.gemwallet.gemstone

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.yield
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.*
import java.net.SocketTimeoutException
import java.util.concurrent.atomic.AtomicInteger

/**
 * Mock provider for testing exception handling in AlienProvider implementations.
 */
class MockProvider(
    private val onRequest: suspend (AlienTarget) -> AlienResponse = {
        AlienResponse(status = 200u, data = ByteArray(0))
    }
) : AlienProvider {
    override suspend fun request(target: AlienTarget): AlienResponse = onRequest(target)
    override fun getEndpoint(chain: Chain): String = "https://mock.endpoint"
}

/**
 * Mock preferences for GemGateway.
 */
class MockPreferences : GemPreferences {
    override fun get(key: String): String? = null
    override fun set(key: String, value: String) {}
    override fun remove(key: String) {}
}

@RunWith(AndroidJUnit4::class)
class GemstoneTest {

    private val permitMessage = """
        {
          "types": {
            "Permit": [
              { "name": "owner", "type": "address" },
              { "name": "spender", "type": "address" },
              { "name": "value", "type": "uint256" },
              { "name": "nonce", "type": "uint256" },
              { "name": "deadline", "type": "uint256" }
            ],
            "EIP712Domain": [
              { "name": "name", "type": "string" },
              { "name": "version", "type": "string" },
              { "name": "chainId", "type": "uint256" },
              { "name": "verifyingContract", "type": "address" }
            ]
          },
          "domain": {
            "name": "USD Coin",
            "version": "2",
            "chainId": "0x1",
            "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
          },
          "primaryType": "Permit",
          "message": {
            "owner": "0x951454cad517fcb54a5a60f20c934df90966b2a7",
            "spender": "0x111111125421ca6dc452d289314280a0f8842a65",
            "value": "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            "nonce": "0",
            "deadline": "1746640317"
          }
        }
    """.trimIndent()

    init {
        System.loadLibrary("gemstone")
    }

    private fun createGateway(provider: AlienProvider): GemGateway {
        return GemGateway(provider, MockPreferences(), MockPreferences(), "https://api.example.com")
    }

    @Test
    fun testLibVersion() {
        assertTrue(libVersion().isNotEmpty())
    }

    /**
     * Test 1: UniFFI-defined exception (AlienException) is caught as GatewayException.
     */
    @Test
    fun testProviderThrowsAlienException() = runBlocking {
        val errorMessage = "Request failed"
        val provider = MockProvider { throw AlienException.RequestException(errorMessage) }
        val gateway = createGateway(provider)

        try {
            gateway.getBalanceCoin("ethereum", "0x1234")
            fail("Expected GatewayException.NetworkException to be thrown")
        } catch (e: GatewayException.NetworkException) {
            assertTrue(e.msg.contains(errorMessage))
        }
    }

    /**
     * Test 2: Standard Java exception becomes InternalException.
     *
     * Note: Unlike AlienException which is properly mapped to GatewayException,
     * standard Java exceptions result in InternalException with UnexpectedUniFFICallbackError.
     */
    @Test
    fun testProviderThrowsStandardException() = runBlocking {
        val errorMessage = "Network timeout"
        val provider = MockProvider { throw SocketTimeoutException(errorMessage) }
        val gateway = createGateway(provider)

        try {
            gateway.getBalanceCoin("ethereum", "0x1234")
            fail("Expected InternalException to be thrown")
        } catch (e: InternalException) {
            assertTrue(e.message?.contains(errorMessage) ?: false)
        }
    }

    @Test
    fun testProviderAsyncResponseAndCancellation() = runBlocking {
        val completedRequests = AtomicInteger()
        val responseData = """{"jsonrpc":"2.0","id":1,"result":"0x"}""".encodeToByteArray()
        val status = GemServiceStatus(MockProvider {
            completedRequests.incrementAndGet()
            AlienResponse(status = 200u, data = responseData)
        })

        repeat(500) {
            status.getEndpointLatency("https://mock.endpoint")
        }
        assertEquals(500, completedRequests.get())

        val simulationClient = WalletConnectSimulationClient(MockProvider { target ->
            assertTrue(target.body?.decodeToString()?.contains("eth_getCode") == true)
            completedRequests.incrementAndGet()
            AlienResponse(status = 200u, data = responseData)
        })
        repeat(500) {
            simulationClient.simulateSignMessage("ethereum", SignDigestType.EIP712, permitMessage, "1inch.io")
            if (it % 50 == 0) {
                System.gc()
            }
        }
        assertEquals(1_000, completedRequests.get())

        val cancellableStatus = GemServiceStatus(MockProvider {
            delay(10)
            AlienResponse(status = 200u, data = responseData)
        })
        repeat(500) {
            val request = launch {
                cancellableStatus.getEndpointLatency("https://mock.endpoint")
            }
            yield()
            request.cancelAndJoin()
        }
        assertEquals(0, uniffiForeignFutureHandleCount())
    }

}
