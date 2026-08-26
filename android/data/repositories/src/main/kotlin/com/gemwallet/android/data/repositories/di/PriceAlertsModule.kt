package com.gemwallet.android.data.repositories.di

import android.content.Context
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepositoryImpl
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import com.gemwallet.android.data.repositories.pricealerts.GemstonePriceAlertStore
import uniffi.gemstone.GemDeviceApiClient
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
    fun provideGemPriceAlertService(apiClient: GemDeviceApiClient, store: GemPriceAlertStore): GemPriceAlertService = GemPriceAlertService(apiClient, store)

    @Provides
    @Singleton
    fun providePriceAlertsRepositoryImpl(
        @ApplicationContext context: Context,
        priceAlertsDao: PriceAlertsDao,
    ): PriceAlertRepository {
        return PriceAlertRepositoryImpl(
            context = context,
            priceAlertsDao = priceAlertsDao,
            configStore = com.gemwallet.android.data.service.store.ConfigStore(
                context.getSharedPreferences(
                    "price-alerts",
                    Context.MODE_PRIVATE
                )
            ),
        )
    }
}