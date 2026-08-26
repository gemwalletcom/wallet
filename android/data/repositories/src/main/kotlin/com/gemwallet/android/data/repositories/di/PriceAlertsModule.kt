package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepositoryImpl
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import com.gemwallet.android.data.repositories.pricealerts.GemstonePriceAlertStore
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemPriceAlertStore
import javax.inject.Singleton

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
    ): GemPriceAlertService = GemPriceAlertService(apiClient, preferencesService, store)

    @Provides
    @Singleton
    fun providePriceAlertsRepositoryImpl(priceAlertsDao: PriceAlertsDao): PriceAlertRepository = PriceAlertRepositoryImpl(priceAlertsDao)
}