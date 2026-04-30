package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.cases.nft.RefreshNftAsset
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.data.repositories.nft.NftRepository
import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
class NftModule {

    @Provides
    @Singleton
    fun provideNftRepository(
        gemDeviceApiClient: GemDeviceApiClient,
        nftDao: NftDao
    ): NftRepository {
        return NftRepository(gemDeviceApiClient, nftDao)
    }

    @Provides
    fun provideSyncNfts(nftRepository: NftRepository): SyncNfts = nftRepository

    @Provides
    fun provideGetNftCase(nftRepository: NftRepository): GetListNftCase = nftRepository

    @Provides
    fun provideGetAssetNftCase(nftRepository: NftRepository): GetAssetNft = nftRepository

    @Provides
    fun provideRefreshNftAsset(nftRepository: NftRepository): RefreshNftAsset = nftRepository
}
