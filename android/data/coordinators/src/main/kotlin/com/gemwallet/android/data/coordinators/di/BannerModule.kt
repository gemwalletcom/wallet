package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.banner.cases.ApplyBannerAction
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.banner.cases.GetBannerContent
import com.gemwallet.android.application.banner.cases.HasMultiSign
import com.gemwallet.android.data.coordinators.banner.ApplyBannerActionImpl
import com.gemwallet.android.data.coordinators.banner.GetActiveBannersImpl
import com.gemwallet.android.data.coordinators.banner.GetBannerContentImpl
import com.gemwallet.android.data.coordinators.banner.HasMultiSignImpl
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.StakeConfig
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BannerModule {

    @Provides
    @Singleton
    fun provideGetActiveBanners(
        getSession: GetSession,
        getAssetInfo: GetAssetInfo,
        bannerStore: GemstoneBannerStore,
        bannerService: GemBannerService,
        stakeConfig: StakeConfig,
    ): GetActiveBanners = GetActiveBannersImpl(getSession, getAssetInfo, bannerStore, bannerService, stakeConfig)

    @Provides
    @Singleton
    fun provideGetBannerContent(bannerService: GemBannerService): GetBannerContent = GetBannerContentImpl(bannerService)

    @Provides
    @Singleton
    fun provideApplyBannerAction(bannerService: GemBannerService): ApplyBannerAction = ApplyBannerActionImpl(bannerService)

    @Provides
    @Singleton
    fun provideHasMultiSign(bannerStore: GemstoneBannerStore): HasMultiSign = HasMultiSignImpl(bannerStore)
}
