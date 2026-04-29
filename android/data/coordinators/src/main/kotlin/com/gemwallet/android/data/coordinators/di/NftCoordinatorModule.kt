package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.nft.coordinators.GetNftAssetDetails
import com.gemwallet.android.application.nft.coordinators.GetNftCollections
import com.gemwallet.android.application.nft.coordinators.RefreshNftAsset
import com.gemwallet.android.application.nft.coordinators.SyncNftCollections
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.cases.nft.RefreshNftAsset as RefreshNftAssetCase
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.coordinators.nft.GetNftAssetDetailsImpl
import com.gemwallet.android.data.coordinators.nft.GetNftCollectionsImpl
import com.gemwallet.android.data.coordinators.nft.RefreshNftAssetImpl
import com.gemwallet.android.data.coordinators.nft.SyncNftCollectionsImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
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
        sessionRepository: SessionRepository,
        getAssetNft: GetAssetNft,
        getCurrentBlockExplorer: GetCurrentBlockExplorer,
    ): GetNftAssetDetails {
        return GetNftAssetDetailsImpl(sessionRepository, getAssetNft, getCurrentBlockExplorer)
    }

    @Provides
    @Singleton
    fun provideGetNftCollections(
        getListNftCase: GetListNftCase,
    ): GetNftCollections {
        return GetNftCollectionsImpl(getListNftCase)
    }

    @Provides
    @Singleton
    fun provideSyncNftCollections(
        sessionRepository: SessionRepository,
        syncNfts: SyncNfts,
    ): SyncNftCollections {
        return SyncNftCollectionsImpl(sessionRepository, syncNfts)
    }

    @Provides
    @Singleton
    fun provideRefreshNftAsset(
        sessionRepository: SessionRepository,
        refreshNftAsset: RefreshNftAssetCase,
    ): RefreshNftAsset {
        return RefreshNftAssetImpl(sessionRepository, refreshNftAsset)
    }
}
