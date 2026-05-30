package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.data.services.gemapi.DeviceKeyPairFixture
import com.gemwallet.android.testkit.mockMulticoinWalletId
import com.wallet.core.primitives.WalletId
import java.util.Base64
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
        var hashedBody: ByteArray? = null
        val signer = DeviceRequestSigner(
            getDeviceId = mockGetDeviceId(),
            bodyHash = { body ->
                hashedBody = body
                "bodyhash"
            },
            signMessage = { _, _ -> "signaturehex" },
            currentTimeMillis = { 123L },
        )
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
        val authorization = signedRequest.header("Authorization")!!
        val payload = String(Base64.getDecoder().decode(authorization.removePrefix("Gem ")))

        assertEquals("${DeviceKeyPairFixture.publicKeyHex}.123.${walletId.id}.bodyhash.signaturehex", payload)
        assertNull(signedRequest.header("x-wallet-id"))
        assertEquals(body, hashedBody!!.toString(Charsets.UTF_8))
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
