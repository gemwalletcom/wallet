package com.gemwallet.android.data.services.nativeprovider

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.Protocol
import okhttp3.Request
import okhttp3.Response
import okhttp3.ResponseBody.Companion.toResponseBody
import org.junit.Assert.assertEquals
import org.junit.Test
import io.mockk.every
import io.mockk.mockk
import uniffi.gemstone.AlienException
import uniffi.gemstone.AlienHttpMethod
import uniffi.gemstone.AlienTarget
import uniffi.gemstone.GemNodeServiceInterface
import java.io.EOFException
import java.io.IOException
import java.net.UnknownHostException

class NativeProviderTest {

    @Test
    fun requestMapsKnownOfflineIoErrors() {
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw UnknownHostException("api.example.com")
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
        } catch (_: AlienException.Offline) {
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
        } catch (_: AlienException.Offline) {
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

    @Test
    fun requestSendsABodylessPostWithAnEmptyBody() {
        var sent: Request? = null
        val provider = nativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor { chain ->
                    sent = chain.request()
                    Response.Builder()
                        .request(chain.request())
                        .protocol(Protocol.HTTP_1_1)
                        .code(200)
                        .message("OK")
                        .body("true".toResponseBody())
                        .build()
                }
                .build(),
        )

        runBlocking {
            provider.request(
                AlienTarget(
                    url = "https://api.gemwallet.com/v2/devices/nft_assets/1/refresh",
                    method = AlienHttpMethod.POST,
                    headers = null,
                    body = null,
                )
            )
        }

        assertEquals("POST", sent?.method)
        assertEquals(0L, sent?.body?.contentLength())
    }

    @Test
    fun getEndpointUsesTheNodeService() {
        val provider = nativeProvider()

        assertEquals("https://gemnodes.com/bitcoin", provider.getEndpoint("bitcoin"))
    }

    private fun nativeProvider(
        httpClient: OkHttpClient = OkHttpClient(),
    ): NativeProvider {
        return NativeProvider(
            nodeService = mockk<GemNodeServiceInterface> {
                every { nodeUrl(any()) } answers { "https://gemnodes.com/${firstArg<String>()}" }
            },
            httpClient = httpClient,
        )
    }
}
