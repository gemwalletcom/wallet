package com.gemwallet.android.data.password

import android.content.Context
import android.util.Log
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.gemwallet.android.application.SecureValueNotFoundException
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
private const val DEVICE_AEAD_KEY_ALIAS = "gem_device_keys_aead_v1"
private const val TAG = "TinkDeviceAuthStore"

private val TINK_DEVICE_KEYS_STORE_CONFIG = TinkStoreConfig(
    preferencesFileName = DEVICE_KEYS_PREFERENCES_FILE_NAME,
    namespace = DEVICE_KEYS_NAMESPACE,
    keysetName = DEVICE_KEYSET_NAME,
    keysetPreferencesFileName = DEVICE_KEYSET_PREFERENCES_FILE_NAME,
    masterKeyAlias = DEVICE_MASTER_KEY_ALIAS,
)

class TinkDeviceAuthStore(
    private val context: Context,
) : SecurityStore<Any> {

    private val Context.dataStore by preferencesDataStore(name = LEGACY_DEVICE_KEYS_DATASTORE_NAME)
    private val tinkAeadProvider = TinkAeadProvider(
        context = context,
        config = TINK_DEVICE_KEYS_STORE_CONFIG,
    )
    private val tinkEncryptedStore = TinkEncryptedKeyValueStore(
        context = context,
        config = TINK_DEVICE_KEYS_STORE_CONFIG,
        aeadProvider = tinkAeadProvider,
    )
    private val encryptedStore = EncryptedKeyValueStore(
        context = context,
        preferencesFileName = DEVICE_KEYS_PREFERENCES_FILE_NAME,
        namespace = DEVICE_KEYS_NAMESPACE,
        aeadProvider = AeadProvider(keyAlias = DEVICE_AEAD_KEY_ALIAS),
        legacyStore = tinkEncryptedStore,
        resetOnInvalidKey = true,
    )

    override suspend fun getValue(key: Any): String = withContext(Dispatchers.IO) {
        val keyValue = key.toString()
        val currentValue = encryptedStore.getString(keyValue)
        if (currentValue != null) {
            return@withContext currentValue
        }

        val value = getLegacyValue(keyValue) ?: throw SecureValueNotFoundException()
        runCatching {
            encryptedStore.putString(keyValue, value)
            removeLegacyValue(keyValue)
        }.onFailure { error ->
            Log.e(TAG, "Keeping legacy device auth value, migration failed", error)
        }
        value
    }

    override suspend fun putValue(key: Any, value: String) = withContext(Dispatchers.IO) {
        val keyValue = key.toString()
        encryptedStore.putString(keyValue, value)
        removeLegacyValue(keyValue)
    }

    private suspend fun getLegacyValue(key: String): String? {
        val storedValue = context.dataStore.data.map { preferences -> preferences[stringPreferencesKey(key)] }
            .firstOrNull() ?: return null
        return try {
            String(tinkAeadProvider.get().decrypt(storedValue.fromHex(), null), UTF_8)
        } catch (error: Exception) {
            if (!(isSecureValueCorruption(error) || isSecureKeyFailure(error))) {
                throw error
            }
            Log.e(TAG, "Ignoring undecryptable legacy device auth value: ${error.javaClass.simpleName}", error)
            null
        }
    }

    private suspend fun removeLegacyValue(key: String) {
        context.dataStore.edit { preferences ->
            preferences.remove(stringPreferencesKey(key))
        }
    }
}
