package com.gemwallet.android.data.adapters.di

import com.gemwallet.android.data.adapters.gemstone.GemstoneBannerStore
import com.gemwallet.android.data.adapters.config.UserConfig
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import android.content.Context
import com.gemwallet.android.data.adapters.gemstone.GemstoneNotificationPermissions
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
    fun provideGemstoneBannerStore(bannersDao: BannersDao): GemstoneBannerStore = GemstoneBannerStore(bannersDao)

    @Singleton
    @Provides
    fun provideGemBannerStore(store: GemstoneBannerStore): GemBannerStore = store

    @Provides
    @Singleton
    fun provideGemNotificationPermissions(@ApplicationContext context: Context): GemNotificationPermissions =
        GemstoneNotificationPermissions(context)

    @Provides
    @Singleton
    fun provideGemBannerService(store: GemBannerStore): GemBannerService =
        GemBannerService(store)
}
