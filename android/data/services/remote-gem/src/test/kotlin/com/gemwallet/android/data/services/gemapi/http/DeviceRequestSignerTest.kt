package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import java.util.Base64
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class DeviceRequestSignerTest {

    @Test
    fun signBuildsExpectedAuthorizationPayload() {
        var hashedBody: ByteArray? = null
        var signedPrivateKeyHex: String? = null
        var signedMessage: ByteArray? = null
        val signer = DeviceRequestSigner(
            getDeviceId = FakeGetDeviceId(
                deviceId = "publickeyhex",
                deviceKey = "privatekeyhex",
            ),
            bodyHash = { body ->
                hashedBody = body
                "bodyhash"
            },
            signMessage = { privateKeyHex, message ->
                signedPrivateKeyHex = privateKeyHex
                signedMessage = message
                "signaturehex"
            },
            currentTimeMillis = { 123L },
        )
        val walletId = "multicoin_0xabc"
        val body = """{"device":"android"}""".toByteArray()

        val signature = signer.sign(
            method = "POST",
            path = "/v2/devices",
            body = body,
            walletId = walletId,
        )

        assertTrue(signature.authorization.startsWith("Gem "))

        val payload = String(Base64.getDecoder().decode(signature.authorization.removePrefix("Gem ")))

        assertEquals("publickeyhex.123.$walletId.bodyhash.signaturehex", payload)
        assertEquals("privatekeyhex", signedPrivateKeyHex)
        assertEquals("123.POST./v2/devices.$walletId.bodyhash", String(signedMessage!!))
        assertArrayEquals(body, hashedBody)
    }
}

private class FakeGetDeviceId(
    private val deviceId: String,
    private val deviceKey: String,
) : GetDeviceId {
    override fun getDeviceId(): String = deviceId

    override fun getDeviceKey(): String = deviceKey
}
