package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneDeviceStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneKeystorePassword
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.data.repositories.addresses.AddressesRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepositoryImpl
import com.gemwallet.android.data.service.store.database.AccountsDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.WalletsDao
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemKeystore
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton
import com.gemwallet.android.data.repositories.gemstone.GemstoneFileStore
import com.gemwallet.android.data.service.store.LocalStore
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemAvatarService
import uniffi.gemstone.GemFileStore
import uniffi.gemstone.GemWalletPreferencesService
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletPreferencesStore

@InstallIn(SingletonComponent::class)
@Module
object WalletsModule {

    @Provides
    fun provideWalletsRepository(
        walletsDao: WalletsDao,
        accountsDao: AccountsDao,
        addressesRepository: AddressesRepository,
        assetsDao: AssetsDao,
        transactionRunner: StoreTransactionRunner,
    ): WalletsRepository {
        return WalletsRepositoryImpl(
            walletsDao = walletsDao,
            accountsDao = accountsDao,
            addressesRepository = addressesRepository,
            assetsDao = assetsDao,
            transactionRunner = transactionRunner,
        )
    }

    @Provides
    @Singleton
    fun provideGemKeystore(@ApplicationContext context: Context): GemKeystore = GemKeystore(context.dataDir.toString())

    @Provides
    @Singleton
    fun provideGemWalletService(
        keystore: GemKeystore,
        passwordStore: PasswordStore,
        walletsRepository: Lazy<WalletsRepository>,
        walletSessionService: GemWalletSessionService,
        deviceStore: GemstoneDeviceStore,
        fileStore: GemFileStore,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemWalletService = GemWalletService(
        keystore,
        GemstoneKeystorePassword(passwordStore),
        GemstoneWalletStore(walletsRepository),
        walletSessionService,
        deviceStore,
        fileStore,
        walletPreferencesService,
    )

    @Provides
    @Singleton
    fun provideGemFileStore(localStore: LocalStore): GemFileStore = GemstoneFileStore(localStore)

    @Provides
    @Singleton
    fun provideGemWalletPreferencesService(@ApplicationContext context: Context): GemWalletPreferencesService =
        GemWalletPreferencesService(GemstoneWalletPreferencesStore(context))

    @Provides
    @Singleton
    fun provideGemAvatarService(
        walletsRepository: Lazy<WalletsRepository>,
        fileStore: GemFileStore,
        alienProvider: AlienProvider,
    ): GemAvatarService = GemAvatarService(
        wallets = GemstoneWalletStore(walletsRepository),
        files = fileStore,
        provider = alienProvider,
    )
}
