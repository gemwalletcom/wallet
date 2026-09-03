package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.device.GemstoneDevicePlatform
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStateStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeveloperService
import uniffi.gemstone.GemDeveloperServiceInterface
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletPreferencesService

@InstallIn(SingletonComponent::class)
@Module
object DeveloperModule {

    @Provides
    fun provideGemDeveloperService(
        platform: GemstoneDevicePlatform,
        preferencesService: GemPreferencesService,
        walletPreferencesService: GemWalletPreferencesService,
        transactionStateStore: GemstoneTransactionStateStore,
        perpetualService: GemPerpetualService,
    ): GemDeveloperServiceInterface =
        GemDeveloperService(platform, preferencesService, walletPreferencesService, transactionStateStore, perpetualService)
}
