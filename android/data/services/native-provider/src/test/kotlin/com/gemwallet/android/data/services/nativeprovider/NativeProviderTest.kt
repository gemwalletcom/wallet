package com.gemwallet.android.data.services.nativeprovider

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.Protocol
import okhttp3.Request
import okhttp3.Response
import okhttp3.ResponseBody.Companion.toResponseBody
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import uniffi.gemstone.AlienException
import uniffi.gemstone.AlienHttpMethod
import uniffi.gemstone.AlienTarget
import uniffi.gemstone.GemApiClient
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWidgetService
import uniffi.gemstone.GemWidgetSize
import java.io.EOFException
import java.io.IOException
import java.net.UnknownHostException

class NativeProviderTest {

    @Test
    fun requestMapsKnownOfflineIoErrors() {
        val provider = NativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw UnknownHostException("api.example.com")
                }
                .build(),
        )

        assertThrows(AlienException.Offline::class.java) {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    ),
                )
            }
        }
    }

    @Test
    fun requestMapsDroppedStreamToOffline() {
        val provider = NativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw IOException("unexpected end of stream on https://gemnodes.com/...", EOFException())
                }
                .build(),
        )

        assertThrows(AlienException.Offline::class.java) {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    ),
                )
            }
        }
    }

    @Test
    fun requestRethrowsCancellation() {
        val provider = NativeProvider(
            httpClient = OkHttpClient.Builder()
                .addInterceptor {
                    throw CancellationException("cancelled")
                }
                .build(),
        )

        val error = assertThrows(CancellationException::class.java) {
            runBlocking {
                provider.request(
                    AlienTarget(
                        url = "https://gemnodes.com/bitcoin",
                        method = AlienHttpMethod.GET,
                        headers = null,
                        body = null,
                    ),
                )
            }
        }
        assertEquals("cancelled", error.message)
    }

    @Test
    fun requestSendsABodylessPostWithAnEmptyBody() {
        var sent: Request? = null
        val provider = NativeProvider(
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
                ),
            )
        }

        assertEquals("POST", sent?.method)
        assertEquals(0L, sent?.body?.contentLength())
    }

    @Test
    fun unexpectedFailureReturnsThroughRust() {
        val provider = NativeProvider(OkHttpClient.Builder().addInterceptor { throw IllegalStateException("interceptor failed") }.build())
        val service = GemWidgetService(GemApiClient(provider))
        val error = assertThrows(GemServiceException.Api::class.java) {
            runBlocking { service.coins(GemWidgetSize.SMALL, "USD") }
        }
        assertEquals("java.lang.IllegalStateException: interceptor failed", error.msg)
    }

    @Test
    fun cancellationReturnsThroughRust() {
        val provider = NativeProvider(OkHttpClient.Builder().addInterceptor { throw CancellationException("callback cancelled") }.build())
        val service = GemWidgetService(GemApiClient(provider))
        val error = assertThrows(GemServiceException.Api::class.java) {
            runBlocking { service.coins(GemWidgetSize.SMALL, "USD") }
        }
        assertEquals("java.util.concurrent.CancellationException: callback cancelled", error.msg)
    }
}
