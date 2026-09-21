package com.gemwallet.android.data.services.gemstone.di

import android.content.Context
import com.gemwallet.android.application.notifications.NotificationPermissionRequests
import com.gemwallet.android.data.service.store.database.BannersDao
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.notifications.GemstoneNotificationPermissions
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBannerStore
import com.gemwallet.android.model.NotificationsAvailable
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemBannerStore
import uniffi.gemstone.GemNotificationPermissions
import uniffi.gemstone.GemPreferencesService
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
    fun provideNotificationPermissionRequests(): NotificationPermissionRequests = NotificationPermissionRequests()

    @Provides
    @Singleton
    fun provideGemNotificationPermissions(@ApplicationContext context: Context, requests: NotificationPermissionRequests, preferences: GemPreferencesService, notificationsAvailable: NotificationsAvailable): GemNotificationPermissions =
        GemstoneNotificationPermissions(context, requests, preferences, notificationsAvailable)

    @Provides
    @Singleton
    fun provideGemBannerService(store: GemBannerStore): GemBannerService = GemBannerService(store)
}
