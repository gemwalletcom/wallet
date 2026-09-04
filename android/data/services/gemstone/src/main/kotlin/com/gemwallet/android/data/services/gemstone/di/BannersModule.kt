package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import android.content.Context
import com.gemwallet.android.data.services.gemstone.notifications.GemstoneNotificationPermissions
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemBannerServiceInterface
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

    @Provides
    fun provideGemBannerServiceInterface(service: GemBannerService): GemBannerServiceInterface = service
}
