package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.banner.cases.GetAssetBanners
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.coordinators.banner.GetAssetBannersImpl
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
    fun provideGetAssetBanners(getSession: GetSession, bannerStore: GemstoneBannerStore): GetAssetBanners = GetAssetBannersImpl(getSession, bannerStore)
}
