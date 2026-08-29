package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import com.gemwallet.android.data.repositories.gemstone.GemstonePriceAlertStore
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemPriceAlertStore
import javax.inject.Singleton
import uniffi.gemstone.GemNotificationPermissions

@InstallIn(SingletonComponent::class)
@Module
object PriceAlertsModule {

    @Singleton
    @Provides
    fun provideGemPriceAlertStore(priceAlertsDao: PriceAlertsDao): GemPriceAlertStore = GemstonePriceAlertStore(priceAlertsDao)

    @Singleton
    @Provides
    fun provideGemPriceAlertService(
        apiClient: GemDeviceApiClient,
        preferencesService: GemPreferencesService,
        store: GemPriceAlertStore,
        deviceService: GemDeviceService,
        notificationPermissions: GemNotificationPermissions,
    ): GemPriceAlertService = GemPriceAlertService(
        api = apiClient,
        preferences = preferencesService,
        store = store,
        device = deviceService,
        permissions = notificationPermissions,
    )
}