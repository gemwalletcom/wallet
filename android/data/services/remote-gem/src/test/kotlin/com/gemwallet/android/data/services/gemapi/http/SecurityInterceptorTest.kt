package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.testkit.mockMulticoinWalletId
import com.wallet.core.primitives.WalletId
import java.security.GeneralSecurityException
import java.util.concurrent.TimeUnit
import okhttp3.Call
import okhttp3.Connection
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.Protocol
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class SecurityInterceptorTest {

    @Test
    fun interceptSignsWalletIdFromRequestTagAndBody() {
        var signedWalletId: String? = null
        var signedBody: ByteArray? = null
        val signer = object : DeviceRequestSigner {
            override fun sign(method: String, path: String, body: ByteArray?, walletId: String): DeviceSignature {
                signedWalletId = walletId
                signedBody = body
                return DeviceSignature("Gem signed")
            }
        }
        val walletId = mockMulticoinWalletId()
        val body = """{"device":"android"}"""
        val request = Request.Builder()
            .url("https://api.gemwallet.com/v2/devices/rewards")
            .post(body.toRequestBody("application/json".toMediaType()))
            .tag(WalletId::class.java, walletId)
            .build()

        val chain = FakeChain(request)
        SecurityInterceptor(signer).intercept(chain)
        val signedRequest = chain.proceededRequest!!

        assertEquals("Gem signed", signedRequest.header("Authorization"))
        assertEquals(walletId.id, signedWalletId)
        assertEquals(body, signedBody!!.toString(Charsets.UTF_8))
        assertNull(signedRequest.header("x-wallet-id"))
    }

    @Test
    fun interceptReturnsPeekableResponseWhenSigningFails() {
        val signer = object : DeviceRequestSigner {
            override fun sign(method: String, path: String, body: ByteArray?, walletId: String): DeviceSignature =
                throw GeneralSecurityException("Keystore operation failed")
        }
        val request = Request.Builder()
            .url("https://api.gemwallet.com/v2/devices/rewards")
            .build()

        val response = SecurityInterceptor(signer).intercept(FakeChain(request))

        assertEquals(503, response.code)
        assertEquals("", response.peekBody(64L * 1024).string())
    }
}

private class FakeChain(
    private val request: Request,
) : Interceptor.Chain {
    var proceededRequest: Request? = null

    override fun request(): Request = request

    override fun proceed(request: Request): Response {
        proceededRequest = request
        return Response.Builder()
            .request(request)
            .protocol(Protocol.HTTP_2)
            .code(200)
            .message("OK")
            .build()
    }

    override fun connection(): Connection? = null

    override fun call(): Call = error("not used")

    override fun connectTimeoutMillis(): Int = 0

    override fun withConnectTimeout(timeout: Int, unit: TimeUnit): Interceptor.Chain = this

    override fun readTimeoutMillis(): Int = 0

    override fun withReadTimeout(timeout: Int, unit: TimeUnit): Interceptor.Chain = this

    override fun writeTimeoutMillis(): Int = 0

    override fun withWriteTimeout(timeout: Int, unit: TimeUnit): Interceptor.Chain = this
}
