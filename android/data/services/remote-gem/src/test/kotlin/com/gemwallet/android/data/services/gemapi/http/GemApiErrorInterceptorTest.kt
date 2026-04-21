package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.data.services.gemapi.GemApiException
import java.util.concurrent.TimeUnit
import okhttp3.Call
import okhttp3.Connection
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.Protocol
import okhttp3.Request
import okhttp3.Response
import okhttp3.ResponseBody.Companion.toResponseBody
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class GemApiErrorInterceptorTest {

    private val interceptor = GemApiErrorInterceptor()
    private val request = Request.Builder()
        .url("https://example.com/v2/devices/rewards/referrals/create")
        .build()

    @Test
    fun `response error body throws gem api exception`() {
        val error = assertThrows(GemApiException::class.java) {
            interceptor.intercept(
                FakeChain(
                    request = request,
                    response = response(
                        code = 500,
                        body = """{"error":{"message":"Username must be at most 16 characters"}}""",
                    ),
                ),
            )
        }

        assertEquals("Username must be at most 16 characters", error.message)
    }

    @Test
    fun `non error response passes through unchanged`() {
        val response = interceptor.intercept(
            FakeChain(
                request = request,
                response = response(
                    code = 200,
                    body = """{"code":"cryptowallet"}""",
                ),
            ),
        )

        assertEquals(200, response.code)
        assertEquals("""{"code":"cryptowallet"}""", response.body.string())
    }

    private fun response(
        code: Int,
        body: String,
    ): Response = Response.Builder()
        .request(request)
        .protocol(Protocol.HTTP_1_1)
        .code(code)
        .message(if (code in 200..299) "OK" else "Error")
        .body(body.toResponseBody("application/json".toMediaType()))
        .build()

    private class FakeChain(
        private val request: Request,
        private val response: Response,
    ) : Interceptor.Chain {

        override fun request(): Request = request

        override fun proceed(request: Request): Response = response

        override fun connection(): Connection? = null

        override fun call(): Call {
            throw UnsupportedOperationException()
        }

        override fun connectTimeoutMillis(): Int = TimeUnit.SECONDS.toMillis(30).toInt()

        override fun withConnectTimeout(timeout: Int, unit: TimeUnit): Interceptor.Chain = this

        override fun readTimeoutMillis(): Int = TimeUnit.SECONDS.toMillis(30).toInt()

        override fun withReadTimeout(timeout: Int, unit: TimeUnit): Interceptor.Chain = this

        override fun writeTimeoutMillis(): Int = TimeUnit.SECONDS.toMillis(30).toInt()

        override fun withWriteTimeout(timeout: Int, unit: TimeUnit): Interceptor.Chain = this
    }
}
