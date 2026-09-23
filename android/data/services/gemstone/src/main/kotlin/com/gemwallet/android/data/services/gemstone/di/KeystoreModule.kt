package com.gemwallet.android.data.services.gemstone.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import com.gemwallet.android.data.services.gemstone.stores.GemstoneSwapStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAuthService
import uniffi.gemstone.GemAvatarService
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemDeviceKeyService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemSignMessageService
import uniffi.gemstone.GemSwapService
import uniffi.gemstone.GemSwapper
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object KeystoreModule {

    @Provides
    @Singleton
    fun provideGemKeystore(@ApplicationContext context: Context): GemKeystore = GemKeystore(context.dataDir.toString())

    @Provides
    @Singleton
    fun provideGemWalletService(
        keystore: GemKeystore,
        passwordStore: PasswordStore,
        walletStore: GemstoneWalletStore,
        walletSessionService: GemWalletSessionService,
        preferencesService: GemPreferencesService,
        fileStore: GemFileStore,
        walletPreferencesService: GemWalletPreferencesService,
        explorerService: GemExplorerService,
        nameService: GemNameService,
        avatarService: GemAvatarService,
    ): GemWalletService = GemWalletService(
        keystore,
        GemstoneKeystorePassword(passwordStore),
        walletStore,
        walletSessionService,
        preferencesService,
        fileStore,
        walletPreferencesService,
        explorerService,
        nameService,
        avatarService,
    )

    @Provides
    @Singleton
    fun provideGemSwapService(swapper: GemSwapper, keystore: GemKeystore, passwordStore: PasswordStore, assetsDao: AssetsDao, transactionsDao: TransactionsDao): GemSwapService = GemSwapService(
        swapper = swapper,
        keystore = keystore,
        password = GemstoneKeystorePassword(passwordStore),
        store = GemstoneSwapStore(assetsDao, transactionsDao),
    )

    @Provides
    fun provideGemSignMessageService(names: GemNameService, explorer: GemExplorerService, keystore: GemKeystore, passwordStore: PasswordStore): GemSignMessageService =
        GemSignMessageService(names, explorer, keystore, GemstoneKeystorePassword(passwordStore))

    @Provides
    @Singleton
    fun provideGemAuthService(apiClient: GemDeviceApiClient, keystore: GemKeystore, passwordStore: PasswordStore, deviceKeyService: GemDeviceKeyService): GemAuthService = GemAuthService(
        apiClient,
        keystore,
        GemstoneKeystorePassword(passwordStore),
        deviceKeyService,
    )
}
