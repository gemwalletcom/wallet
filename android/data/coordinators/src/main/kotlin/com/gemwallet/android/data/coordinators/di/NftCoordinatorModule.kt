package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemCollectibleServiceInterface
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.nft.cases.GetAssetNft
import com.gemwallet.android.data.services.gemstone.stores.GemstoneNftStore
import com.gemwallet.android.application.nft.cases.GetListNft
import com.gemwallet.android.data.coordinators.nft.GetNftAssetDetailsImpl
import com.gemwallet.android.data.coordinators.nft.GetNftCollectionsImpl
import com.gemwallet.android.application.session.cases.GetSession
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object NftCoordinatorModule {

    @Provides
    @Singleton
    fun provideGetNftAssetDetails(
        getSession: GetSession,
        getAssetNft: GetAssetNft,
        nftStore: GemstoneNftStore,
        collectibleService: GemCollectibleServiceInterface,
    ): GetNftAssetDetails {
        return GetNftAssetDetailsImpl(getSession, getAssetNft, nftStore, collectibleService)
    }

    @Provides
    @Singleton
    fun provideGetNftCollections(
        getSession: GetSession,
        getListNftCase: GetListNft,
    ): GetNftCollections {
        return GetNftCollectionsImpl(getSession, getListNftCase)
    }
}
