package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemNftService
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.nft.cases.RefreshNftAsset
import com.gemwallet.android.application.nft.cases.SyncNftCollections
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.data.coordinators.nft.GetNftAssetDetailsImpl
import com.gemwallet.android.data.coordinators.nft.GetNftCollectionsImpl
import com.gemwallet.android.data.coordinators.nft.RefreshNftAssetImpl
import com.gemwallet.android.data.coordinators.nft.SyncNftCollectionsImpl
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
        explorerService: GemExplorerService,
    ): GetNftAssetDetails {
        return GetNftAssetDetailsImpl(getSession, getAssetNft, explorerService)
    }

    @Provides
    @Singleton
    fun provideGetNftCollections(
        getSession: GetSession,
        getListNftCase: GetListNftCase,
    ): GetNftCollections {
        return GetNftCollectionsImpl(getSession, getListNftCase)
    }

    @Provides
    @Singleton
    fun provideSyncNftCollections(
        nftService: GemNftService,
    ): SyncNftCollections {
        return SyncNftCollectionsImpl(nftService)
    }

    @Provides
    @Singleton
    fun provideRefreshNftAsset(
        getSession: GetSession,
        nftService: GemNftService,
    ): RefreshNftAsset {
        return RefreshNftAssetImpl(getSession, nftService)
    }
}
