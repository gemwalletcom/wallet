package com.gemwallet.android.data.password

import com.google.crypto.tink.Aead
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Assert.assertThrows
import org.junit.Test
import java.security.GeneralSecurityException
import java.security.InvalidKeyException
import java.security.ProviderException
import javax.crypto.AEADBadTagException

class TinkAeadRetryTest {

    private val aead = object : Aead {
        override fun encrypt(plaintext: ByteArray?, associatedData: ByteArray?): ByteArray = ByteArray(0)

        override fun decrypt(ciphertext: ByteArray?, associatedData: ByteArray?): ByteArray = ByteArray(0)
    }

    @Test
    fun retriesTransientKeystoreFailures() {
        var attempts = 0

        val result = retryAeadBuild(retryDelayMilliseconds = 0) {
            attempts += 1
            when (attempts) {
                1 -> throw InvalidKeyException("Keystore cannot load the key")
                2 -> throw ProviderException("Keystore busy")
                else -> aead
            }
        }

        assertSame(aead, result)
        assertEquals(3, attempts)
    }

    @Test
    fun doesNotRetryUnrelatedFailure() {
        var attempts = 0

        val error = assertThrows(IllegalStateException::class.java) {
            retryAeadBuild(retryDelayMilliseconds = 0) {
                attempts += 1
                throw IllegalStateException("not a keystore failure")
            }
        }

        assertEquals("not a keystore failure", error.message)
        assertEquals(1, attempts)
    }

    @Test
    fun doesNotRetryInvalidAuthenticationTag() {
        var attempts = 0

        val error = assertThrows(AEADBadTagException::class.java) {
            retryAeadBuild(retryDelayMilliseconds = 0) {
                attempts += 1
                throw AEADBadTagException("Invalid authentication tag")
            }
        }

        assertEquals("Invalid authentication tag", error.message)
        assertEquals(1, attempts)
    }

    @Test
    fun rethrowsLastErrorWhenAllAttemptsFail() {
        var attempts = 0

        val error = assertThrows(GeneralSecurityException::class.java) {
            retryAeadBuild(retryDelayMilliseconds = 0) {
                attempts += 1
                throw GeneralSecurityException("attempt $attempts")
            }
        }

        assertEquals("attempt 3", error.message)
        assertEquals(3, attempts)
    }
}
