package com.gemwallet.android.services

import android.content.Context
import coil3.ImageLoader
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.security.cases.KeystoreRecovery
import com.gemwallet.android.data.password.SecureStorageRecovery
import com.gemwallet.android.data.password.TinkPasswordStore
import com.gemwallet.android.data.services.gemstone.config.AppLockPreferences
import com.gemwallet.android.data.services.gemstone.device.DevicePushSettings
import com.gemwallet.android.data.services.gemstone.stores.GemstonePreferencesStore
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.first
import okhttp3.OkHttpClient
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

class KeystoreRecoveryService @Inject constructor(
    private val walletsQuery: WalletsQuery,
    private val walletService: GemWalletServiceInterface,
    private val passwords: TinkPasswordStore,
    private val migrateV3KeystoreService: MigrateV3KeystoreService,
    private val database: GemDatabase,
    private val httpClient: OkHttpClient,
    private val imageLoader: ImageLoader,
    private val secureStorage: SecureStorageRecovery,
    @param:ApplicationContext private val context: Context,
) : KeystoreRecovery {

    override suspend fun repairLockStores(): Boolean {
        if (!passwords.canReadPasswords(walletPasswordKeys())) return false
        secureStorage.clearUnreadableLockStores()
        return true
    }

    override suspend fun resetLocalData() {
        walletsQuery().first().forEach { walletService.deleteWallet(it.id.id) }
        database.clearAllTables()
        preferencesFileNames.forEach { name ->
            check(context.getSharedPreferences(name, Context.MODE_PRIVATE).edit().clear().commit()) { "Preferences reset failed" }
        }
        httpClient.cache?.evictAll()
        imageLoader.diskCache?.clear()
        secureStorage.clear()
    }

    private suspend fun walletPasswordKeys(): List<String> = walletsQuery().first()
        .filter { it.type != WalletType.View }
        .map { wallet -> if (migrateV3KeystoreService.needsMigration(wallet)) wallet.id.id else PasswordStore.Keys.Password.key }
        .distinct()

    private companion object {
        val preferencesFileNames = listOf(
            GemstonePreferencesStore.APP_PREFERENCES_FILE_NAME,
            GemstonePreferencesStore.GATEWAY_PREFERENCES_FILE_NAME,
            DevicePushSettings.PREFERENCES_FILE_NAME,
            AppLockPreferences.CONFIG_FILE_NAME,
        )
    }
}
