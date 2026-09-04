package com.gemwallet.android.data.services.nativeprovider

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
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
