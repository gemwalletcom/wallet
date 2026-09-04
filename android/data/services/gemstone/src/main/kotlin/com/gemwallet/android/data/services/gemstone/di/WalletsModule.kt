package com.gemwallet.android.data.services.gemstone.di

import dagger.hilt.android.qualifiers.ApplicationContext
import android.content.Context
import uniffi.gemstone.GemLocalizer
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import com.gemwallet.android.data.service.store.database.AccountsDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.WalletsDao
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton
import com.gemwallet.android.data.services.gemstone.GemstoneFileStore
import com.gemwallet.android.data.service.store.LocalStore
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemAvatarService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletPreferencesService
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletPreferencesStore

@InstallIn(SingletonComponent::class)
@Module
object WalletsModule {

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
        addressStore: GemstoneAddressStore,
        localizer: GemLocalizer,
    ): GemWalletService = GemWalletService(
        keystore,
        GemstoneKeystorePassword(passwordStore),
        walletStore,
        walletSessionService,
        preferencesService,
        fileStore,
        walletPreferencesService,
        explorerService,
        addressStore,
        localizer,
    )

    @Provides
    fun provideGemWalletServiceInterface(service: GemWalletService): GemWalletServiceInterface = service

    @Provides
    @Singleton
    fun provideGemFileStore(localStore: LocalStore): GemFileStore = GemstoneFileStore(localStore)

    @Provides
    @Singleton
    fun provideGemWalletPreferencesService(@ApplicationContext context: Context): GemWalletPreferencesService =
        GemWalletPreferencesService(GemstoneWalletPreferencesStore(context))

    @Provides
    @Singleton
    fun provideGemWalletStore(
        walletsDao: WalletsDao,
        accountsDao: AccountsDao,
        assetsDao: AssetsDao,
        transactionRunner: StoreTransactionRunner,
    ): GemstoneWalletStore = GemstoneWalletStore(walletsDao, accountsDao, assetsDao, transactionRunner)

    @Provides
    @Singleton
    fun provideGemAvatarService(
        walletStore: GemstoneWalletStore,
        fileStore: GemFileStore,
        alienProvider: AlienProvider,
    ): GemAvatarService = GemAvatarService(
        wallets = walletStore,
        files = fileStore,
        provider = alienProvider,
    )
}
