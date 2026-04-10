package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.application.asset_select.coordinators.SearchSelectAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.data.coordinators.asset_select.GetRecentAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.GetSelectAssetsInfoImpl
import com.gemwallet.android.data.coordinators.asset_select.SearchSelectAssetsImpl
import com.gemwallet.android.data.coordinators.asset_select.SwitchAssetVisibilityImpl
import com.gemwallet.android.data.coordinators.asset_select.ToggleAssetPinImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
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
        assetsRepository: AssetsRepository,
    ): SearchSelectAssets = SearchSelectAssetsImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetSelectAssetsInfo(
        assetsRepository: AssetsRepository,
    ): GetSelectAssetsInfo = GetSelectAssetsInfoImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideGetRecentAssets(
        assetsRepository: AssetsRepository,
    ): GetRecentAssets = GetRecentAssetsImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideSwitchAssetVisibility(
        assetsRepository: AssetsRepository,
    ): SwitchAssetVisibility = SwitchAssetVisibilityImpl(assetsRepository)

    @Provides
    @Singleton
    fun provideToggleAssetPin(
        assetsRepository: AssetsRepository,
    ): ToggleAssetPin = ToggleAssetPinImpl(assetsRepository)
}
