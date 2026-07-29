package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.asset_select.coordinators.ClearRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.coordinators.SearchListAssets
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.coordinators.asset_select.ClearRecentAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.GetRecentAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.GetSelectAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset_select.SearchListAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.SearchSelectAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.SwitchAssetVisibilityImpl
import com.gemwallet.android.data.coordinators.asset_select.UpdateRecentAssetImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.AssetsSearchService
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AssetSelectModule {

    @Provides
    @Singleton
    fun provideSearchSelectAssets(
        searchService: AssetsSearchService,
    ): SearchSelectAssets = SearchSelectAssetsImpl(searchService)

    @Provides
    @Singleton
    fun provideSearchListAssets(
        searchService: AssetsSearchService,
    ): SearchListAssets = SearchListAssetsImpl(searchService)

    @Provides
    @Singleton
    fun provideGetSelectAssetsInfo(
        assetsRepository: AssetsRepository,
    ): GetSelectAssetsInfo = GetSelectAssetsInfoImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetRecentAssets(
        recentAssetsService: RecentAssetsService,
    ): GetRecentAssets = GetRecentAssetsImpl(recentAssetsService)

    @Provides
    @Singleton
    fun provideSwitchAssetVisibility(
        enableAsset: EnableAsset,
        assetsRepository: AssetsRepository,
    ): SwitchAssetVisibility = SwitchAssetVisibilityImpl(enableAsset, assetsRepository)

    @Provides
    @Singleton
    fun provideUpdateRecentAsset(
        sessionRepository: SessionRepository,
        recentAssetsService: RecentAssetsService,
    ): UpdateRecentAsset = UpdateRecentAssetImpl(sessionRepository, recentAssetsService)

    @Provides
    @Singleton
    fun provideClearRecentAssets(
        sessionRepository: SessionRepository,
        recentAssetsService: RecentAssetsService,
    ): ClearRecentAssets = ClearRecentAssetsImpl(sessionRepository, recentAssetsService)
}
