package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.nft.cases.GetAssetNft
import com.gemwallet.android.application.nft.cases.GetListNft
import com.gemwallet.android.data.coordinators.nft.GetAssetNftImpl
import com.gemwallet.android.data.coordinators.nft.GetListNftImpl
import com.gemwallet.android.data.services.gemstone.stores.GemstoneNftStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemNftService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object NftCasesModule {

    @Provides
    @Singleton
    fun provideGetListNft(nftStore: GemstoneNftStore): GetListNft = GetListNftImpl(nftStore)

    @Provides
    @Singleton
    fun provideGetAssetNft(nftService: GemNftService, nftStore: GemstoneNftStore): GetAssetNft = GetAssetNftImpl(nftService, nftStore)
}
