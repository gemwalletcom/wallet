package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.data.coordinators.pricealerts.GetPriceAlertsImpl
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceAlertStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PriceAlertModule {

    @Provides
    @Singleton
    fun provideGetPriceAlerts(priceAlertStore: GemstonePriceAlertStore, getWalletAssets: GetWalletAssets, priceAlertFormatter: PriceAlertFormatter): GetPriceAlerts = GetPriceAlertsImpl(
        priceAlertStore = priceAlertStore,
        getWalletAssets = getWalletAssets,
        priceAlertFormatter = priceAlertFormatter,
    )
}
