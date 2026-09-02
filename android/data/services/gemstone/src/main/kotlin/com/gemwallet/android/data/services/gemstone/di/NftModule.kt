package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneNftStore
import com.gemwallet.android.data.service.store.database.NftDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAvatarService
import uniffi.gemstone.GemCollectibleService
import uniffi.gemstone.GemCollectibleServiceInterface
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemNftService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
class NftModule {

    @Provides
    @Singleton
    fun provideGemstoneNftStore(nftDao: NftDao): GemstoneNftStore = GemstoneNftStore(nftDao)

    @Provides
    @Singleton
    fun provideGemNftService(apiClient: GemDeviceApiClient, nftStore: GemstoneNftStore): GemNftService =
        GemNftService(apiClient, nftStore)

    @Provides
    fun provideGemCollectibleService(nfts: GemNftService, avatars: GemAvatarService, explorer: GemExplorerService): GemCollectibleServiceInterface =
        GemCollectibleService(nfts, avatars, explorer)
}
