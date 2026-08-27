package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.gemstone.GemstoneNotificationStore
import com.gemwallet.android.data.repositories.notifications.InAppNotificationsRepository
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNotificationService
import uniffi.gemstone.GemNotificationStore
import javax.inject.Singleton
import uniffi.gemstone.GemWalletPreferencesService

@InstallIn(SingletonComponent::class)
@Module
object InAppNotificationsModule {

    @Provides
    @Singleton
    fun provideGemNotificationStore(
        notificationsDao: InAppNotificationsDao,
    ): GemNotificationStore = GemstoneNotificationStore(notificationsDao)

    @Provides
    @Singleton
    fun provideGemNotificationService(
        apiClient: GemDeviceApiClient,
        store: GemNotificationStore,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemNotificationService = GemNotificationService(apiClient, store, walletPreferencesService)

    @Provides
    @Singleton
    fun provideInAppNotificationsRepository(
        notificationService: GemNotificationService,
        notificationsDao: InAppNotificationsDao,
    ): InAppNotificationsRepository = InAppNotificationsRepository(
        notificationService = notificationService,
        notificationsDao = notificationsDao,
    )
}
