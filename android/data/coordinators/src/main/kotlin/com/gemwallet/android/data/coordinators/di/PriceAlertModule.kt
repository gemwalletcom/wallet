package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.pricealerts.cases.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.cases.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.cases.HasAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.cases.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.cases.SetAssetPriceAlertEnabled
import com.gemwallet.android.application.pricealerts.cases.SetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.cases.SyncAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.cases.UpdatePriceAlerts
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceAlertStore
import com.gemwallet.android.data.coordinators.pricealerts.GetAssetPriceAlertStateImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.HasAssetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.PriceAlertsCoordinator
import com.gemwallet.android.data.coordinators.pricealerts.SyncAssetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.UpdatePriceAlertsImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.PriceAlertFormatter
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PriceAlertModule {

    @Provides
    @Singleton
    fun providePriceAlertsCoordinator(
        priceAlertService: GemPriceAlertService,
        getCurrentCurrency: GetCurrentCurrency,
    ): PriceAlertsCoordinator = PriceAlertsCoordinator(
        priceAlertService = priceAlertService,
        getCurrentCurrency = getCurrentCurrency,
    )

    @Provides
    fun provideGetPriceAlertsEnabled(coordinator: PriceAlertsCoordinator): GetPriceAlertsEnabled = coordinator

    @Provides
    fun provideSetPriceAlertsEnabled(coordinator: PriceAlertsCoordinator): SetPriceAlertsEnabled = coordinator

    @Provides
    fun provideIncludePriceAlert(coordinator: PriceAlertsCoordinator): IncludePriceAlert = coordinator

    @Provides
    fun provideExcludePriceAlert(coordinator: PriceAlertsCoordinator): ExcludePriceAlert = coordinator

    @Provides
    fun provideSetAssetPriceAlertEnabled(coordinator: PriceAlertsCoordinator): SetAssetPriceAlertEnabled = coordinator

    @Provides
    @Singleton
    fun provideGetPriceAlerts(
        priceAlertStore: GemstonePriceAlertStore,
        getWalletAssets: GetWalletAssets,
        priceAlertFormatter: PriceAlertFormatter,
    ): GetPriceAlerts = GetPriceAlertsImpl(
        priceAlertStore = priceAlertStore,
        getWalletAssets = getWalletAssets,
        priceAlertFormatter = priceAlertFormatter,
    )

    @Provides
    @Singleton
    fun provideAssetPriceAlertState(
        priceAlertStore: GemstonePriceAlertStore,
    ): GetAssetPriceAlertState = GetAssetPriceAlertStateImpl(priceAlertStore)

    @Provides
    fun provideUpdatePriceAlerts(
        priceAlertService: GemPriceAlertService,
    ): UpdatePriceAlerts = UpdatePriceAlertsImpl(priceAlertService = priceAlertService)

    @Provides
    @Singleton
    fun provideHasAssetPriceAlerts(
        priceAlertStore: GemstonePriceAlertStore,
    ): HasAssetPriceAlerts = HasAssetPriceAlertsImpl(priceAlertStore)

    @Provides
    fun provideSyncAssetPriceAlerts(
        hasAssetPriceAlerts: HasAssetPriceAlerts,
        updatePriceAlerts: UpdatePriceAlerts,
    ): SyncAssetPriceAlerts = SyncAssetPriceAlertsImpl(
        hasAssetPriceAlerts = hasAssetPriceAlerts,
        updatePriceAlerts = updatePriceAlerts,
    )
}
