package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.gemstone.GemstoneNotificationStore
import com.gemwallet.android.data.repositories.notifications.InAppNotificationsRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNotificationService
import uniffi.gemstone.GemNotificationStore
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object InAppNotificationsModule {

    @Provides
    @Singleton
    fun provideGemNotificationStore(
        notificationsDao: InAppNotificationsDao,
        walletPreferencesFactory: WalletPreferencesFactory,
    ): GemNotificationStore = GemstoneNotificationStore(notificationsDao, walletPreferencesFactory)

    @Provides
    @Singleton
    fun provideGemNotificationService(
        apiClient: GemDeviceApiClient,
        store: GemNotificationStore,
    ): GemNotificationService = GemNotificationService(apiClient, store)

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
