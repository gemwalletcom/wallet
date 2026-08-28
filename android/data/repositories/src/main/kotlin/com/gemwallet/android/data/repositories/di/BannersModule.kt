package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.cases.banners.BannerActionCase
import com.gemwallet.android.cases.banners.GetBannersCase
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.banners.BannersRepository
import com.gemwallet.android.data.repositories.gemstone.GemstoneBannerStore
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import android.content.Context
import com.gemwallet.android.data.repositories.gemstone.GemstoneNotificationPermissions
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemNotificationPermissions
import uniffi.gemstone.GemBannerStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BannersModule {
    @Singleton
    @Provides
    fun provideGemBannerStore(bannersDao: BannersDao): GemBannerStore = GemstoneBannerStore(bannersDao)

    @Provides
    @Singleton
    fun provideGemNotificationPermissions(@ApplicationContext context: Context): GemNotificationPermissions =
        GemstoneNotificationPermissions(context)

    @Provides
    @Singleton
    fun provideGemBannerService(store: GemBannerStore): GemBannerService =
        GemBannerService(store)

    @Provides
    @Singleton
    fun provideBannersRepository(
        assetsRepository: AssetsRepository,
        bannersDao: BannersDao,
        bannerService: GemBannerService,
    ): BannersRepository {
        return BannersRepository(
            assetsRepository,
            bannersDao,
            bannerService,
        )
    }

    @Singleton
    @Provides
    fun provideGetBannersCase(bannersRepository: BannersRepository): GetBannersCase = bannersRepository

    @Singleton
    @Provides
    fun provideBannerActionCase(bannersRepository: BannersRepository): BannerActionCase = bannersRepository

    @Singleton
    @Provides
    fun provideGetWalletOperationsEnabled(bannersRepository: BannersRepository): HasMultiSign = bannersRepository
}
