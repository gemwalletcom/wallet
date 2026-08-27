package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepositoryImpl
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import com.gemwallet.android.data.repositories.gemstone.GemstonePriceAlertStore
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemPriceAlertStore
import javax.inject.Singleton
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.repositories.gemstone.GemstoneDeviceSync
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
        syncDevice: SyncDevice,
        notificationPermissions: GemNotificationPermissions,
    ): GemPriceAlertService = GemPriceAlertService(
        api = apiClient,
        preferences = preferencesService,
        store = store,
        device = GemstoneDeviceSync(syncDevice),
        permissions = notificationPermissions,
    )

    @Provides
    @Singleton
    fun providePriceAlertsRepositoryImpl(priceAlertsDao: PriceAlertsDao): PriceAlertRepository = PriceAlertRepositoryImpl(priceAlertsDao)
}