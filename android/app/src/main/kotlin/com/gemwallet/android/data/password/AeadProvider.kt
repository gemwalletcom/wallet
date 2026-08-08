package com.gemwallet.android.data.password

import com.google.crypto.tink.Aead
import com.google.crypto.tink.integration.android.AndroidKeystore
import java.security.InvalidKeyException

internal class AeadProvider(
    private val keyAlias: String,
) {

    private var aead: Aead? = null

    fun encrypt(
        plaintext: ByteArray,
        associatedData: ByteArray,
        createKeyIfMissing: Boolean,
    ): ByteArray = synchronized(this) {
        get(createKeyIfMissing).encrypt(plaintext, associatedData)
    }

    fun decrypt(ciphertext: ByteArray, associatedData: ByteArray): ByteArray = synchronized(this) {
        get(createKeyIfMissing = false).decrypt(ciphertext, associatedData)
    }

    fun refresh() {
        synchronized(this) {
            aead = null
        }
    }

    fun reset() {
        synchronized(this) {
            aead = null
            synchronized(ANDROID_KEYSTORE_LOCK) {
                AndroidKeystore.deleteKey(keyAlias)
            }
        }
    }

    private fun get(createKeyIfMissing: Boolean): Aead =
        aead ?: createAead(createKeyIfMissing).also { aead = it }

    private fun createAead(createKeyIfMissing: Boolean): Aead {
        synchronized(ANDROID_KEYSTORE_LOCK) {
            if (!AndroidKeystore.hasKey(keyAlias)) {
                if (!createKeyIfMissing) {
                    throw InvalidKeyException("Android Keystore key is missing: $keyAlias")
                }
                AndroidKeystore.generateNewAes256GcmKey(keyAlias)
            }
            return AndroidKeystore.getAead(keyAlias)
        }
    }
}

internal val ANDROID_KEYSTORE_LOCK = Any()
