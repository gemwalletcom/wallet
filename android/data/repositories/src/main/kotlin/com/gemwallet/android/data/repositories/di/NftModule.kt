package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.gemstone.GemstoneNftStore
import com.gemwallet.android.data.service.store.database.NftDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNftRulesService
import uniffi.gemstone.GemNftService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
class NftModule {

    @Provides
    @Singleton
    fun provideGemNftService(apiClient: GemDeviceApiClient, nftDao: NftDao): GemNftService =
        GemNftService(apiClient, GemstoneNftStore(nftDao))

    @Provides
    @Singleton
    fun provideGemNftRulesService(): GemNftRulesService = GemNftRulesService()
}
