package com.gemwallet.android.data.password

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.application.SecurityStore
import com.gemwallet.android.math.fromHex
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import java.nio.charset.StandardCharsets.UTF_8

private const val LEGACY_DEVICE_KEYS_DATASTORE_NAME = "device_keys"
private const val DEVICE_KEYSET_NAME = "ngen_gem_keyset"
private const val DEVICE_KEYSET_PREFERENCES_FILE_NAME = "gem_device_master_key"
private const val DEVICE_MASTER_KEY_ALIAS = "gem_device_master_key"
private const val DEVICE_KEYS_PREFERENCES_FILE_NAME = "gem_device_keys"
private const val DEVICE_KEYS_NAMESPACE = "device_keys"

private val DEVICE_KEYS_STORE_CONFIG = TinkStoreConfig(
    preferencesFileName = DEVICE_KEYS_PREFERENCES_FILE_NAME,
    namespace = DEVICE_KEYS_NAMESPACE,
    keysetName = DEVICE_KEYSET_NAME,
    keysetPreferencesFileName = DEVICE_KEYSET_PREFERENCES_FILE_NAME,
    masterKeyAlias = DEVICE_MASTER_KEY_ALIAS,
)

class TinkSecurityStore(
    private val context: Context,
) : SecurityStore<Any> {

    private val Context.dataStore by preferencesDataStore(name = LEGACY_DEVICE_KEYS_DATASTORE_NAME)
    private val aeadProvider = TinkAeadProvider(
        context = context,
        config = DEVICE_KEYS_STORE_CONFIG,
    )
    private val encryptedStore = TinkEncryptedKeyValueStore(
        context = context,
        config = DEVICE_KEYS_STORE_CONFIG,
        aeadProvider = aeadProvider,
    )

    override suspend fun getValue(key: Any): String = withContext(Dispatchers.IO) {
        val keyValue = key.toString()
        val currentValue = encryptedStore.getString(keyValue)
        if (currentValue != null) {
            return@withContext currentValue
        }

        val value = getLegacyValue(keyValue) ?: throw IllegalStateException("Data not found")
        encryptedStore.putString(keyValue, value)
        removeLegacyValue(keyValue)
        value
    }

    override suspend fun putValue(key: Any, value: String) = withContext(Dispatchers.IO) {
        val keyValue = key.toString()
        encryptedStore.putString(keyValue, value)
        removeLegacyValue(keyValue)
    }

    private suspend fun getLegacyValue(key: String): String? {
        return context.dataStore.data.map { preferences -> preferences[stringPreferencesKey(key)] }
            .firstOrNull()?.let {
                String(aeadProvider.get().decrypt(it.fromHex(), null), UTF_8)
            }
    }

    private suspend fun removeLegacyValue(key: String) {
        context.dataStore.edit { preferences ->
            preferences.remove(stringPreferencesKey(key))
        }
    }
}
