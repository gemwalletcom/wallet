@file:Suppress("DEPRECATION")

package com.gemwallet.android.data.password

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.google.crypto.tink.aead.AeadConfig
import com.google.crypto.tink.aead.AesGcmKeyManager
import com.google.crypto.tink.daead.AesSivKeyManager
import com.google.crypto.tink.daead.DeterministicAeadConfig
import com.google.crypto.tink.integration.android.AndroidKeystoreKmsClient

// Passwords and Gemstone secure preferences historically shared this file; their keys must stay disjoint.
internal const val LEGACY_PREFERENCES_FILE_NAME = "pwd"
private const val ANDROIDX_KEY_KEYSET_ALIAS = "__androidx_security_crypto_encrypted_prefs_key_keyset__"
private const val ANDROIDX_VALUE_KEYSET_ALIAS = "__androidx_security_crypto_encrypted_prefs_value_keyset__"

internal class LegacyEncryptedPreferences(context: Context, private val preferencesFileName: String) : SecureStringStore {

    private val context = context.applicationContext

    @Volatile
    private var sharedPreferences: SharedPreferences? = null

    override fun contains(key: String): Boolean = synchronized(secureStorageLock) { existingPreferences()?.contains(key) == true }

    override fun getString(key: String): String? = synchronized(secureStorageLock) { existingPreferences()?.getString(key, null) }

    override fun putString(key: String, value: String): Unit = synchronized(secureStorageLock) {
        if (!preferences().edit().putString(key, value).commit()) {
            throw IllegalStateException("Legacy secure value write failed")
        }
    }

    override fun removeString(key: String): Boolean = synchronized(secureStorageLock) { existingPreferences()?.edit()?.remove(key)?.commit() != false }

    private fun preferences(): SharedPreferences {
        sharedPreferences?.let { return it }
        return synchronized(secureStorageLock) {
            sharedPreferences ?: createPreferences().also { sharedPreferences = it }
        }
    }

    private fun existingPreferences(): SharedPreferences? {
        if (sharedPreferences == null && context.securePreferences(preferencesFileName).all.isEmpty()) {
            return null
        }
        return preferences()
    }

    private fun createPreferences(): SharedPreferences {
        val rawPreferences = context.securePreferences(preferencesFileName)
        val isEmpty = rawPreferences.all.isEmpty()
        check(isEmpty || (rawPreferences.contains(ANDROIDX_KEY_KEYSET_ALIAS) && rawPreferences.contains(ANDROIDX_VALUE_KEYSET_ALIAS))) { "Legacy secure keyset is missing" }
        AeadConfig.register()
        DeterministicAeadConfig.register()
        val masterAead = AndroidKeystoreKmsClient.getOrGenerateNewAeadKey("android-keystore://${MasterKey.DEFAULT_MASTER_KEY_ALIAS}")
        encryptedKeyset(rawPreferences, ANDROIDX_KEY_KEYSET_ALIAS, masterAead, AesSivKeyManager.aes256SivTemplate().takeIf { isEmpty })
        encryptedKeyset(rawPreferences, ANDROIDX_VALUE_KEYSET_ALIAS, masterAead, AesGcmKeyManager.aes256GcmTemplate().takeIf { isEmpty })
        val masterKey = MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        return EncryptedSharedPreferences.create(
            context,
            preferencesFileName,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }
}
