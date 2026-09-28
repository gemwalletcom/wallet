package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.StakeDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneStakeStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemAmountService
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemStakeService
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeStore
import uniffi.gemstone.GemStaticApiClient
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.Platform
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeModule {
    @Singleton
    @Provides
    fun provideGemstoneStakeStore(stakeDao: StakeDao, assetsDao: AssetsDao): GemstoneStakeStore = GemstoneStakeStore(stakeDao, assetsDao)

    @Singleton
    @Provides
    fun provideGemStakeStore(store: GemstoneStakeStore): GemStakeStore = store

    @Singleton
    @Provides
    fun provideGemStakeService(
        gateway: GemGateway,
        staticApiClient: GemStaticApiClient,
        store: GemStakeStore,
        nameService: GemNameService,
        explorerService: GemExplorerService,
        preferencesService: GemPreferencesService,
        walletSessionService: GemWalletSessionService,
    ): GemStakeService = GemStakeService(gateway, staticApiClient, store, nameService, explorerService, preferencesService, walletSessionService, Platform.ANDROID)

    @Provides
    @Singleton
    fun provideGemStakeServiceInterface(service: GemStakeService): GemStakeServiceInterface = service

    @Provides
    fun provideGemAmountService(stake: GemStakeService, preferences: GemPreferencesService, walletSessionService: GemWalletSessionService): GemAmountServiceInterface = GemAmountService(stake, preferences, walletSessionService)
}
