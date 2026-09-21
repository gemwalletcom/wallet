package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneNotificationStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNotificationService
import uniffi.gemstone.GemNotificationServiceInterface
import uniffi.gemstone.GemNotificationStore
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object InAppNotificationsModule {

    @Provides
    @Singleton
    fun provideGemstoneNotificationStore(notificationsDao: InAppNotificationsDao): GemstoneNotificationStore = GemstoneNotificationStore(notificationsDao)

    @Provides
    @Singleton
    fun provideGemNotificationStore(store: GemstoneNotificationStore): GemNotificationStore = store

    @Provides
    @Singleton
    fun provideGemNotificationService(apiClient: GemDeviceApiClient, store: GemNotificationStore, walletPreferencesService: GemWalletPreferencesService, walletSessionService: GemWalletSessionService): GemNotificationService =
        GemNotificationService(apiClient, store, walletPreferencesService, walletSessionService)

    @Provides
    fun provideGemNotificationServiceInterface(service: GemNotificationService): GemNotificationServiceInterface = service
}
