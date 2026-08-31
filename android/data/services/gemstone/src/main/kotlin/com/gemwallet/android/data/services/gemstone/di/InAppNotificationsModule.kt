package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneNotificationStore
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
    fun provideGemstoneNotificationStore(
        notificationsDao: InAppNotificationsDao,
    ): GemstoneNotificationStore = GemstoneNotificationStore(notificationsDao)

    @Provides
    @Singleton
    fun provideGemNotificationStore(store: GemstoneNotificationStore): GemNotificationStore = store

    @Provides
    @Singleton
    fun provideGemNotificationService(
        apiClient: GemDeviceApiClient,
        store: GemNotificationStore,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemNotificationService = GemNotificationService(apiClient, store, walletPreferencesService)
}
