package com.gemwallet.android.data.services.gemapi

import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Node
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.Protocol
import okhttp3.Response
import okhttp3.ResponseBody.Companion.toResponseBody
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.AlienException
import uniffi.gemstone.AlienHttpMethod
import uniffi.gemstone.AlienTarget
import java.io.EOFException
import java.io.IOException
import java.net.UnknownHostException

class NativeProviderTest {

    @Test
    fun requestCachesByPrivateHeaderAndStripsIt() {
        var calls = 0
        var forwardedCacheHeader: String? = null
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor { chain ->
                    calls += 1
                    forwardedCacheHeader = chain.request().header(NATIVE_PROVIDER_CACHE_HEADER)
                    Response.Builder()
                        .request(chain.request())
                        .protocol(Protocol.HTTP_1_1)
                        .code(200)
                        .message("OK")
                        .body("response-$calls".toResponseBody())
                        .build()
                }
                .build(),
        )
        val target = AlienTarget(
            url = "https://gemnodes.com/info",
            method = AlienHttpMethod.GET,
            headers = mapOf(
                "accept" to "application/json",
                NATIVE_PROVIDER_CACHE_HEADER to "60",
            ),
            body = null,
        )

        val first = runBlocking { provider.request(target) }
        val second = runBlocking { provider.request(target) }

        assertEquals("response-1", first.data.decodeToString())
        assertEquals("response-1", second.data.decodeToString())
        assertEquals(1, calls)
        assertNull(forwardedCacheHeader)
    }

    @Test
    fun requestMapsKnownOfflineIoErrors() {
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw UnknownHostException("api.example.com")
                }
                .build(),
            config = NativeProviderConfig(networkOfflineMessage = "offline"),
        )

        try {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    )
                )
            }
        } catch (err: AlienException.RequestException) {
            assertEquals("offline", err.msg)
            return
        }
        throw AssertionError("Expected offline request exception")
    }

    @Test
    fun requestMapsDroppedStreamToOffline() {
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw IOException("unexpected end of stream on https://gemnodes.com/...", EOFException())
                }
                .build(),
            config = NativeProviderConfig(networkOfflineMessage = "offline"),
        )

        try {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    )
                )
            }
        } catch (err: AlienException.RequestException) {
            assertEquals("offline", err.msg)
            return
        }
        throw AssertionError("Expected request exception")
    }

    @Test
    fun requestRethrowsCancellation() {
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw CancellationException("cancelled")
                }
                .build(),
        )

        try {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    )
                )
            }
        } catch (err: CancellationException) {
            assertEquals("cancelled", err.message)
            return
        }
        throw AssertionError("Expected cancellation exception")
    }

    private fun nativeProvider(
        httpClient: OkHttpClient = OkHttpClient(),
        config: NativeProviderConfig = NativeProviderConfig(networkOfflineMessage = "offline"),
    ): NativeProvider {
        return NativeProvider(
            getNodesCase = object : GetNodesCase {
                override suspend fun getNodes(chain: Chain): Flow<List<Node>> = emptyFlow()
            },
            getCurrentNodeCase = object : GetCurrentNodeCase {
                override fun getCurrentNode(chain: Chain): Node? = null
            },
            setCurrentNodeCase = object : SetCurrentNodeCase {
                override fun setCurrentNode(chain: Chain, node: Node) = Unit
            },
            httpClient = httpClient,
            config = config,
        )
    }
}
