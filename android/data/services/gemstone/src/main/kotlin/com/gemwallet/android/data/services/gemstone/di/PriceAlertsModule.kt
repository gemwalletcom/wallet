package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceAlertStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemNotificationPermissions
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemPriceAlertServiceInterface
import uniffi.gemstone.GemPriceAlertStore
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PriceAlertsModule {

    @Singleton
    @Provides
    fun provideGemstonePriceAlertStore(priceAlertsDao: PriceAlertsDao, priceAlertFormatter: PriceAlertFormatter): GemstonePriceAlertStore = GemstonePriceAlertStore(priceAlertsDao, priceAlertFormatter)

    @Provides
    @Singleton
    fun provideGemPriceAlertStore(store: GemstonePriceAlertStore): GemPriceAlertStore = store

    @Singleton
    @Provides
    fun provideGemPriceAlertService(apiClient: GemDeviceApiClient, preferencesService: GemPreferencesService, store: GemPriceAlertStore, notificationPermissions: GemNotificationPermissions): GemPriceAlertService = GemPriceAlertService(
        api = apiClient,
        preferences = preferencesService,
        store = store,
        permissions = notificationPermissions,
    )

    @Provides
    fun provideGemPriceAlertServiceInterface(service: GemPriceAlertService): GemPriceAlertServiceInterface = service
}
