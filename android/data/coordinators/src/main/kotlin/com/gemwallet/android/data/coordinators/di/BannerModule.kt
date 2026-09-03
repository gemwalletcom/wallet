package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.banner.cases.HasMultiSign
import com.gemwallet.android.data.coordinators.banner.GetActiveBannersImpl
import com.gemwallet.android.data.coordinators.banner.HasMultiSignImpl
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BannerModule {

    @Provides
    @Singleton
    fun provideGetActiveBanners(
        getSession: GetSession,
        getAssetInfo: GetAssetInfo,
        getActiveAssetsInfo: GetActiveAssetsInfo,
        bannerStore: GemstoneBannerStore,
    ): GetActiveBanners = GetActiveBannersImpl(getSession, getAssetInfo, getActiveAssetsInfo, bannerStore)

    @Provides
    @Singleton
    fun provideHasMultiSign(bannerStore: GemstoneBannerStore): HasMultiSign = HasMultiSignImpl(bannerStore)
}
