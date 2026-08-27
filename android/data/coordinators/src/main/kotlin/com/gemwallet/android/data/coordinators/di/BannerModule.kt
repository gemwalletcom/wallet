package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.banner.coordinators.ApplyBannerAction
import com.gemwallet.android.application.banner.coordinators.GetActiveBanners
import com.gemwallet.android.cases.banners.BannerActionCase
import com.gemwallet.android.cases.banners.GetBannersCase
import com.gemwallet.android.data.coordinators.banner.ApplyBannerActionImpl
import com.gemwallet.android.data.coordinators.banner.GetActiveBannersImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
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
        sessionRepository: SessionRepository,
        getBannersCase: GetBannersCase,
    ): GetActiveBanners {
        return GetActiveBannersImpl(sessionRepository, getBannersCase)
    }

    @Provides
    @Singleton
    fun provideApplyBannerAction(
        bannerActionCase: BannerActionCase,
    ): ApplyBannerAction {
        return ApplyBannerActionImpl(bannerActionCase)
    }
}
