package com.gemwallet.android.data.password

import android.content.Context
import androidx.datastore.preferences.core.edit
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject

class SecureStorageRecovery @Inject constructor(@param:ApplicationContext private val context: Context) {

    fun clearUnreadableLockStores() = synchronized(secureStorageLock) {
        if (LegacyEncryptedPreferences(context, LEGACY_PREFERENCES_FILE_NAME).isKeyChanged()) {
            clear(preferencesFiles = listOf(LEGACY_PREFERENCES_FILE_NAME), keyAliases = listOf(LEGACY_MASTER_KEY_ALIAS))
        }
        if (TinkAeadProvider(context, GEM_PREFERENCES_STORE_CONFIG).isKeyChanged()) {
            clear(preferencesFiles = GEM_PREFERENCES_STORE_CONFIG.preferencesFiles, keyAliases = listOf(GEM_PREFERENCES_STORE_CONFIG.masterKeyAlias))
        }
    }

    suspend fun clear() {
        context.legacyDeviceKeys.edit { it.clear() }
        synchronized(secureStorageLock) {
            val stores = listOf(PASSWORD_STORE_CONFIG, GEM_PREFERENCES_STORE_CONFIG, DEVICE_KEYS_STORE_CONFIG)
            clear(
                preferencesFiles = stores.flatMap { it.preferencesFiles } + LEGACY_PREFERENCES_FILE_NAME,
                keyAliases = stores.map { it.masterKeyAlias } + PASSWORD_STORE_CONFIG.authenticatedMasterKeyAlias + LEGACY_MASTER_KEY_ALIAS,
            )
        }
    }

    private fun clear(preferencesFiles: List<String>, keyAliases: List<String>) {
        preferencesFiles.forEach { name -> check(context.getSharedPreferences(name, Context.MODE_PRIVATE).edit().clear().commit()) { "Secure storage reset failed" } }
        val keyStore = androidKeyStore()
        keyAliases.forEach(keyStore::deleteEntry)
    }
}
