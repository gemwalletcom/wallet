package com.gemwallet.android.services

import android.content.Context
import coil3.ImageLoader
import com.gemwallet.android.data.password.SecureStorageReset
import com.gemwallet.android.data.services.gemstone.config.AppLockPreferences
import com.gemwallet.android.data.services.gemstone.device.DevicePushSettings
import com.gemwallet.android.data.services.gemstone.stores.GemstonePreferencesStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.data.services.store.database.GemDatabase
import dagger.hilt.android.qualifiers.ApplicationContext
import okhttp3.OkHttpClient
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

class LocalDataReset @Inject constructor(
    private val walletStore: GemstoneWalletStore,
    private val walletService: GemWalletServiceInterface,
    private val database: GemDatabase,
    private val httpClient: OkHttpClient,
    private val imageLoader: ImageLoader,
    private val secureStorage: SecureStorageReset,
    @param:ApplicationContext private val context: Context,
) {
    suspend fun reset() {
        walletStore.getWallets().forEach { walletService.deleteWallet(it.id) }
        database.clearAllTables()
        preferencesFileNames.forEach { name ->
            check(context.getSharedPreferences(name, Context.MODE_PRIVATE).edit().clear().commit()) { "Preferences reset failed" }
        }
        httpClient.cache?.evictAll()
        imageLoader.diskCache?.clear()
        secureStorage.clear()
    }

    private companion object {
        val preferencesFileNames = listOf(
            GemstonePreferencesStore.APP_PREFERENCES_FILE_NAME,
            GemstonePreferencesStore.GATEWAY_PREFERENCES_FILE_NAME,
            DevicePushSettings.PREFERENCES_FILE_NAME,
            AppLockPreferences.CONFIG_FILE_NAME,
        )
    }
}
