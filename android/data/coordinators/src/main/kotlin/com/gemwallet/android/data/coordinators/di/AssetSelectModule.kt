package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.asset_select.cases.GetChainAssets
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.application.asset_select.cases.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.cases.SearchListAssets
import com.gemwallet.android.application.asset_select.cases.SearchSelectAssets
import com.gemwallet.android.data.coordinators.asset_select.GetChainAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.GetRecentAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.GetSelectAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset_select.SearchListAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.SearchSelectAssetsImpl
import com.gemwallet.android.data.services.gemstone.assets.AssetsSearchService
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.assets.cases.SyncBalances
import com.gemwallet.android.application.assets.cases.GetWalletAssets

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
    fun provideGetChainAssets(
        assetStore: GemstoneAssetStore,
        getCurrentWalletId: GetCurrentWalletId,
        syncBalances: SyncBalances,
    ): GetChainAssets = GetChainAssetsImpl(assetStore, getCurrentWalletId, syncBalances)

    @Provides
    @Singleton
    fun provideGetSelectAssetsInfo(
        getWalletAssets: GetWalletAssets,
    ): GetSelectAssetsInfo = GetSelectAssetsInfoImpl(getWalletAssets)

    @Provides
    @Singleton
    fun provideGetRecentAssets(
        recentAssetsService: RecentAssetsService,
    ): GetRecentAssets = GetRecentAssetsImpl(recentAssetsService)
}
