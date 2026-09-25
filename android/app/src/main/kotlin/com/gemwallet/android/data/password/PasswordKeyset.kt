package com.gemwallet.android.data.password

import android.content.Context
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.hex
import com.google.crypto.tink.Aead
import com.google.crypto.tink.KeysetHandle
import com.google.crypto.tink.RegistryConfiguration
import com.google.crypto.tink.TinkProtoKeysetFormat
import com.google.crypto.tink.aead.AeadConfig
import com.google.crypto.tink.integration.android.AndroidKeystoreKmsClient
import java.security.KeyStore

internal class PasswordKeyset(context: Context, private val config: TinkStoreConfig) {
    private val context = context.applicationContext
    private val preferences by lazy { this.context.securePreferences(config.keysetPreferencesFileName) }
    private val protectedName = "${config.keysetName}_authenticated"
    private val authenticated = AuthenticatedKeysetAead("${config.masterKeyAlias}_authenticated")
    private val unprotected = TinkAeadProvider(context, config)

    val authenticationRequired: Boolean
        get() = preferences.contains(protectedName)

    fun get(): Aead = if (authenticationRequired) protectedHandle().getPrimitive(RegistryConfiguration.get(), Aead::class.java) else unprotected.get()

    @Synchronized
    fun setAuthenticationRequired(required: Boolean) {
        if (required == authenticationRequired) return
        AeadConfig.register()
        if (required) {
            val handle = unprotected.keysetHandle()
            authenticated.createKey()
            val encrypted = try {
                TinkProtoKeysetFormat.serializeEncryptedKeyset(handle, authenticated, byteArrayOf())
            } catch (error: Exception) {
                authenticated.deleteKey()
                throw error
            }
            check(preferences.edit().putString(protectedName, encrypted.hex).remove(config.keysetName).commit()) { "Wallet keyset write failed" }
            KeyStore.getInstance("AndroidKeyStore").apply { load(null) }.deleteEntry(config.masterKeyAlias)
        } else {
            val handle = protectedHandle()
            val master = AndroidKeystoreKmsClient.getOrGenerateNewAeadKey("android-keystore://${config.masterKeyAlias}")
            val encrypted = TinkProtoKeysetFormat.serializeEncryptedKeyset(handle, master, byteArrayOf())
            check(preferences.edit().putString(config.keysetName, encrypted.hex).remove(protectedName).commit()) { "Wallet keyset write failed" }
            authenticated.deleteKey()
        }
        unprotected.reset()
    }

    private fun protectedHandle(): KeysetHandle {
        AeadConfig.register()
        val bytes = checkNotNull(preferences.getString(protectedName, null)) { "Wallet keyset is missing" }.fromHex()
        return try {
            TinkProtoKeysetFormat.parseEncryptedKeyset(bytes, authenticated, byteArrayOf())
        } finally {
            bytes.fill(0)
        }
    }
}
