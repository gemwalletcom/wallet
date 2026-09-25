package com.gemwallet.android.data.password

import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import com.google.crypto.tink.Aead
import java.security.GeneralSecurityException
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

internal const val PASSWORD_AUTHENTICATION_WINDOW_SECONDS = 30

internal class AuthenticatedKeysetAead(private val alias: String) : Aead {
    private val keyStore: KeyStore
        get() = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    fun keyExists(): Boolean = keyStore.containsAlias(alias)

    fun deleteKey() = keyStore.deleteEntry(alias)

    fun createKey() {
        if (keyStore.containsAlias(alias)) return
        val builder = KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
            .setKeySize(256)
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setUserAuthenticationRequired(true)
            .setInvalidatedByBiometricEnrollment(false)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            builder.setUserAuthenticationParameters(PASSWORD_AUTHENTICATION_WINDOW_SECONDS, KeyProperties.AUTH_BIOMETRIC_STRONG or KeyProperties.AUTH_DEVICE_CREDENTIAL)
        } else {
            builder.setUserAuthenticationValidityDurationSeconds(PASSWORD_AUTHENTICATION_WINDOW_SECONDS)
        }
        KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").apply {
            init(builder.build())
            generateKey()
        }
    }

    override fun encrypt(plaintext: ByteArray, associatedData: ByteArray): ByteArray {
        val cipher = cipher()
        cipher.init(Cipher.ENCRYPT_MODE, key())
        cipher.updateAAD(associatedData)
        return cipher.iv + cipher.doFinal(plaintext)
    }

    override fun decrypt(ciphertext: ByteArray, associatedData: ByteArray): ByteArray {
        if (ciphertext.size < IV_SIZE + TAG_SIZE / 8) throw GeneralSecurityException("Invalid wallet keyset")
        val cipher = cipher()
        cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(TAG_SIZE, ciphertext, 0, IV_SIZE))
        cipher.updateAAD(associatedData)
        return cipher.doFinal(ciphertext, IV_SIZE, ciphertext.size - IV_SIZE)
    }

    private fun key(): SecretKey = keyStore.getKey(alias, null) as? SecretKey ?: throw GeneralSecurityException("Wallet authentication key is missing")

    private fun cipher(): Cipher = Cipher.getInstance("AES/GCM/NoPadding")

    private companion object {
        const val IV_SIZE = 12
        const val TAG_SIZE = 128
    }
}
