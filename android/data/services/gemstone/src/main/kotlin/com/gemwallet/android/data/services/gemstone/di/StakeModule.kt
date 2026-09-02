package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneStakeStore
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StakeDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAmountService
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemAddressStore
import uniffi.gemstone.GemStakeService
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeStore
import uniffi.gemstone.GemStaticApiClient
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object StakeModule {
    @Singleton
    @Provides
    fun provideGemstoneStakeStore(stakeDao: StakeDao, assetsDao: AssetsDao): GemstoneStakeStore =
        GemstoneStakeStore(stakeDao, assetsDao)

    @Singleton
    @Provides
    fun provideGemStakeStore(store: GemstoneStakeStore): GemStakeStore = store

    @Singleton
    @Provides
    fun provideGemStakeService(
        gateway: GemGateway,
        staticApiClient: GemStaticApiClient,
        store: GemStakeStore,
        addressStore: GemAddressStore,
        explorerService: GemExplorerService,
        preferencesService: GemPreferencesService,
        walletSessionService: GemWalletSessionService,
    ): GemStakeService = GemStakeService(gateway, staticApiClient, store, addressStore, explorerService, preferencesService, walletSessionService)

    @Provides
    @Singleton
    fun provideGemStakeServiceInterface(service: GemStakeService): GemStakeServiceInterface = service

    @Provides
    fun provideGemAmountService(stake: GemStakeService, preferences: GemPreferencesService): GemAmountServiceInterface =
        GemAmountService(stake, preferences)
}
