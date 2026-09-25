package com.gemwallet.android.data.password

import android.content.Context
import com.gemwallet.android.math.hex
import com.google.crypto.tink.Aead
import com.google.crypto.tink.KeysetHandle
import com.google.crypto.tink.RegistryConfiguration
import com.google.crypto.tink.aead.AeadConfig
import com.google.crypto.tink.aead.AesGcmKeyManager
import com.google.crypto.tink.integration.android.AndroidKeystoreKmsClient
import java.nio.charset.StandardCharsets.UTF_8
import java.security.MessageDigest
import java.util.Base64

internal class TinkEncryptedKeyValueStore(context: Context, private val config: TinkStoreConfig, private val aeadProvider: () -> Aead) : SecureStringStore {

    private val sharedPreferences by lazy { context.applicationContext.securePreferences(config.preferencesFileName) }

    override fun contains(key: String): Boolean = synchronized(secureStorageLock) { sharedPreferences.contains(storageKey(key)) }

    override fun getString(key: String): String? = synchronized(secureStorageLock) {
        val encryptedValue = sharedPreferences.getString(storageKey(key), null) ?: return@synchronized null
        val decryptedValue = aeadProvider().decrypt(Base64.getDecoder().decode(encryptedValue), associatedData(key))
        String(decryptedValue, UTF_8)
    }

    override fun putString(key: String, value: String): Unit = synchronized(secureStorageLock) {
        val encryptedValue = aeadProvider().encrypt(value.toByteArray(UTF_8), associatedData(key))
        val encodedValue = Base64.getEncoder().encodeToString(encryptedValue)
        if (!sharedPreferences.edit().putString(storageKey(key), encodedValue).commit()) {
            throw IllegalStateException("Secure value write failed")
        }
    }

    override fun removeString(key: String): Boolean = synchronized(secureStorageLock) { sharedPreferences.edit().remove(storageKey(key)).commit() }

    private fun associatedData(key: String): ByteArray = "${config.namespace}:$key".toByteArray(UTF_8)

    private fun storageKey(key: String): String {
        val digest = MessageDigest.getInstance("SHA-256").digest("${config.namespace}\u0000$key".toByteArray(UTF_8))
        return "${config.namespace}_${digest.hex}"
    }

    companion object {
        fun create(context: Context, config: TinkStoreConfig): TinkEncryptedKeyValueStore = TinkEncryptedKeyValueStore(
            context = context,
            config = config,
            aeadProvider = TinkAeadProvider(context = context, config = config)::get,
        )
    }
}

internal data class TinkStoreConfig(val preferencesFileName: String, val namespace: String, val keysetName: String, val keysetPreferencesFileName: String, val masterKeyAlias: String)

internal class TinkAeadProvider(context: Context, private val config: TinkStoreConfig, private val masterAead: () -> Aead = { AndroidKeystoreKmsClient.getOrGenerateNewAeadKey("android-keystore://${config.masterKeyAlias}") }) {

    private val context = context.applicationContext

    @Volatile
    private var aead: Aead? = null

    fun get(): Aead {
        aead?.let { return it }
        return synchronized(secureStorageLock) {
            aead ?: keysetHandle().getPrimitive(RegistryConfiguration.get(), Aead::class.java).also { aead = it }
        }
    }

    fun reset() {
        aead = null
    }

    fun keysetHandle(): KeysetHandle {
        AeadConfig.register()
        val preferences = context.securePreferences(config.keysetPreferencesFileName)
        val values = context.securePreferences(config.preferencesFileName)
        return encryptedKeyset(
            preferences = preferences,
            name = config.keysetName,
            masterAead = masterAead(),
            template = AesGcmKeyManager.aes256GcmTemplate().takeIf { values.all.isEmpty() },
        )
    }
}
