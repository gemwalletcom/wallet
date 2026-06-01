package com.gemwallet.android.data.services.gemapi.http

import com.gemwallet.android.data.services.gemapi.DeviceKeyPairFixture
import com.gemwallet.android.testkit.mockMulticoinWalletId
import java.util.Base64
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class DeviceRequestSignerTest {

    @Test
    fun signBuildsExpectedAuthorizationPayload() {
        var hashedBody: ByteArray? = null
        var signedPublicKeyHex: String? = null
        var signedMessage: ByteArray? = null
        val signer = DeviceRequestSigner(
            getDeviceId = mockGetDeviceId(),
            bodyHash = { body ->
                hashedBody = body
                "bodyhash"
            },
            signMessage = { deviceKeyPair, message ->
                signedPublicKeyHex = deviceKeyPair.publicKeyHex
                signedMessage = message
                "signaturehex"
            },
            currentTimeMillis = { 123L },
        )
        val walletId = mockMulticoinWalletId().id
        val body = """{"device":"android"}""".toByteArray()

        val signature = signer.sign(
            method = "POST",
            path = "/v2/devices",
            body = body,
            walletId = walletId,
        )

        assertTrue(signature.authorization.startsWith("Gem "))

        val payload = String(Base64.getDecoder().decode(signature.authorization.removePrefix("Gem ")))

        assertEquals("${DeviceKeyPairFixture.publicKeyHex}.123.$walletId.bodyhash.signaturehex", payload)
        assertEquals(DeviceKeyPairFixture.publicKeyHex, signedPublicKeyHex)
        assertEquals("123.POST./v2/devices.$walletId.bodyhash", String(signedMessage!!))
        assertArrayEquals(body, hashedBody)
    }

    @Test
    fun signUsesEd25519AndSha256() {
        val walletId = mockMulticoinWalletId().id
        val signer = mockDeviceRequestSigner()

        val signature = signer.sign(
            method = "POST",
            path = "/v2/devices",
            body = """{"device":"android"}""".toByteArray(),
            walletId = walletId,
        )

        val payload = String(Base64.getDecoder().decode(signature.authorization.removePrefix("Gem ")))
        val expectedPayload = listOf(
            DeviceKeyPairFixture.publicKeyHex,
            "123",
            walletId,
            "f7f90abc33e204d8a7f7821efc63eae3f72a0513161b92d61156528860f1d75b",
            "0435dc3dbeff5d054e09d990ffb10d000d50d7a9e92f98da69ad4d57d4ee503c38aa48d9f0a329be7a564c56e0a58e2de1c9a408c356fe5ff075a997fe2e1b0b",
        ).joinToString(".")

        assertEquals(expectedPayload, payload)
    }
}
