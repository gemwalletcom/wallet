package com.gemwallet.android.data.password

import android.content.Context
import androidx.datastore.preferences.core.edit
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject

class SecureStorageReset @Inject constructor(@param:ApplicationContext private val context: Context) {

    suspend fun clear() {
        context.legacyDeviceKeys.edit { it.clear() }
        synchronized(secureStorageLock) {
            val stores = listOf(PASSWORD_STORE_CONFIG, GEM_PREFERENCES_STORE_CONFIG, DEVICE_KEYS_STORE_CONFIG)
            val preferencesFiles = stores.flatMap { listOf(it.preferencesFileName, it.keysetPreferencesFileName) } + LEGACY_PREFERENCES_FILE_NAME
            preferencesFiles.forEach { name -> check(context.getSharedPreferences(name, Context.MODE_PRIVATE).edit().clear().commit()) { "Secure storage reset failed" } }
            val keyStore = androidKeyStore()
            (stores.map { it.masterKeyAlias } + PASSWORD_STORE_CONFIG.authenticatedMasterKeyAlias + LEGACY_MASTER_KEY_ALIAS).forEach(keyStore::deleteEntry)
        }
    }
}
