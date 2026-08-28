package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.nft.cases.RefreshNftAsset
import com.gemwallet.android.application.nft.cases.SyncNftCollections
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.cases.nft.GetListNftCase
import com.gemwallet.android.cases.nft.RefreshNftAsset as RefreshNftAssetCase
import com.gemwallet.android.cases.nft.SyncNfts
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
        explorerService: GemExplorerService,
    ): GetNftAssetDetails {
        return GetNftAssetDetailsImpl(sessionRepository, getAssetNft, explorerService)
    }

    @Provides
    @Singleton
    fun provideGetNftCollections(
        sessionRepository: SessionRepository,
        getListNftCase: GetListNftCase,
    ): GetNftCollections {
        return GetNftCollectionsImpl(sessionRepository, getListNftCase)
    }

    @Provides
    @Singleton
    fun provideSyncNftCollections(
        syncNfts: SyncNfts,
    ): SyncNftCollections {
        return SyncNftCollectionsImpl(syncNfts)
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
