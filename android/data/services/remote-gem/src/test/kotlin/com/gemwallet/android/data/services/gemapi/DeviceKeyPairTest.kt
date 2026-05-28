package com.gemwallet.android.data.services.gemapi

import com.gemwallet.android.math.fromHex
import com.google.crypto.tink.subtle.Ed25519Verify
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class DeviceKeyPairTest {

    @Test
    fun generateCreatesVerifiableEd25519KeyPair() {
        val keyPair = DeviceKeyPair.generate()
        val publicKey = keyPair.publicKeyHex.fromHex()

        val signature = keyPair.sign(DeviceKeyPairFixture.deviceAuthMessage).fromHex()

        Ed25519Verify(publicKey).verify(signature, DeviceKeyPairFixture.deviceAuthMessage)
        assertEquals(64, keyPair.privateKeyHex.length)
        assertEquals(64, keyPair.publicKeyHex.length)
    }

    @Test
    fun fromHexUsesDecodedEd25519PrivateKey() {
        val keyPair = DeviceKeyPair.fromHex(DeviceKeyPairFixture.privateKeyHex)
        val publicKey = DeviceKeyPairFixture.publicKeyHex.fromHex()

        val signatureHex = keyPair.sign(DeviceKeyPairFixture.deviceAuthMessage)

        assertEquals(DeviceKeyPairFixture.privateKeyHex, keyPair.privateKeyHex)
        assertEquals(DeviceKeyPairFixture.publicKeyHex, keyPair.publicKeyHex)
        assertEquals(DeviceKeyPairFixture.deviceAuthSignatureHex, signatureHex)
        Ed25519Verify(publicKey).verify(signatureHex.fromHex(), DeviceKeyPairFixture.deviceAuthMessage)
    }

    @Test
    fun fromHexRejectsInvalidPrivateKeyHex() {
        assertThrows(IllegalArgumentException::class.java) {
            DeviceKeyPair.fromHex("abcd")
        }
    }
}
