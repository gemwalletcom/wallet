package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.banner.cases.ApplyBannerAction
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.coordinators.banner.ApplyBannerActionImpl
import com.gemwallet.android.data.coordinators.banner.GetActiveBannersImpl
import com.gemwallet.android.data.coordinators.banner.HasMultiSignImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.BannersDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemBannerService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BannerModule {

    @Provides
    @Singleton
    fun provideGetActiveBanners(
        sessionRepository: SessionRepository,
        getAssetInfo: GetAssetInfo,
        bannersDao: BannersDao,
        bannerService: GemBannerService,
    ): GetActiveBanners = GetActiveBannersImpl(sessionRepository, getAssetInfo, bannersDao, bannerService)

    @Provides
    @Singleton
    fun provideApplyBannerAction(bannerService: GemBannerService): ApplyBannerAction = ApplyBannerActionImpl(bannerService)

    @Provides
    @Singleton
    fun provideHasMultiSign(bannersDao: BannersDao): HasMultiSign = HasMultiSignImpl(bannersDao)
}
