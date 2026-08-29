package com.gemwallet.android.data.coordinators.di

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
import com.gemwallet.android.data.coordinators.pricealerts.ExcludePriceAlertImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetAssetPriceAlertStateImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.HasAssetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.IncludePriceAlertImpl
import com.gemwallet.android.data.coordinators.pricealerts.SetAssetPriceAlertEnabledImpl
import com.gemwallet.android.data.coordinators.pricealerts.PriceAlertsEnabledCoordinator
import com.gemwallet.android.data.coordinators.pricealerts.SyncAssetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.UpdatePriceAlertsImpl
import com.gemwallet.android.data.repositories.gemstone.GemstonePriceAlertStore
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPriceAlertService
import javax.inject.Singleton
import com.gemwallet.android.application.assets.cases.GetWalletAssets

@InstallIn(SingletonComponent::class)
@Module
object PriceAlertModule {
    @Provides
    @Singleton
    fun provideAddPriceAlerts(
        priceAlertService: GemPriceAlertService,
        getCurrentCurrency: GetCurrentCurrency,
        setPriceAlertsEnabled: SetPriceAlertsEnabled,
    ): IncludePriceAlert {
        return IncludePriceAlertImpl(
            priceAlertService = priceAlertService,
            getCurrentCurrency = getCurrentCurrency,
            setPriceAlertsEnabled = setPriceAlertsEnabled,
        )
    }

    @Provides
    @Singleton
    fun provideGetPriceAlerts(
        priceAlertStore: GemstonePriceAlertStore,
        getWalletAssets: GetWalletAssets,
    ): GetPriceAlerts {
        return GetPriceAlertsImpl(
            priceAlertStore = priceAlertStore,
            getWalletAssets = getWalletAssets,
        )
    }

    @Provides
    @Singleton
    fun providePriceAlertsEnabledCoordinator(
        priceAlertService: GemPriceAlertService,
    ): PriceAlertsEnabledCoordinator = PriceAlertsEnabledCoordinator(priceAlertService)

    @Provides
    fun provideGetPriceAlertsEnabled(coordinator: PriceAlertsEnabledCoordinator): GetPriceAlertsEnabled = coordinator

    @Provides
    fun provideSetPriceAlertsEnabled(coordinator: PriceAlertsEnabledCoordinator): SetPriceAlertsEnabled = coordinator

    @Provides
    @Singleton
    fun provideSetAssetPriceAlertEnabled(
        includePriceAlert: IncludePriceAlert,
        excludePriceAlert: ExcludePriceAlert,
    ): SetAssetPriceAlertEnabled {
        return SetAssetPriceAlertEnabledImpl(
            includePriceAlert = includePriceAlert,
            excludePriceAlert = excludePriceAlert,
        )
    }

    @Provides
    @Singleton
    fun providePriceAlertExclude(
        priceAlertService: GemPriceAlertService,
        getCurrentCurrency: GetCurrentCurrency,
    ): ExcludePriceAlert {
        return ExcludePriceAlertImpl(
            priceAlertService = priceAlertService,
            getCurrentCurrency = getCurrentCurrency,
        )
    }

    @Provides
    @Singleton
    fun provideAssetPriceAlertState(
        priceAlertStore: GemstonePriceAlertStore,
    ): GetAssetPriceAlertState = GetAssetPriceAlertStateImpl(priceAlertStore)

    @Provides
    fun provideUpdatePriceAlerts(
        priceAlertService: GemPriceAlertService,
    ): UpdatePriceAlerts {
        return UpdatePriceAlertsImpl(
            priceAlertService = priceAlertService,
        )
    }

    @Provides
    @Singleton
    fun provideHasAssetPriceAlerts(
        priceAlertStore: GemstonePriceAlertStore,
    ): HasAssetPriceAlerts = HasAssetPriceAlertsImpl(priceAlertStore)

    @Provides
    fun provideSyncAssetPriceAlerts(
        hasAssetPriceAlerts: HasAssetPriceAlerts,
        updatePriceAlerts: UpdatePriceAlerts,
    ): SyncAssetPriceAlerts {
        return SyncAssetPriceAlertsImpl(
            hasAssetPriceAlerts = hasAssetPriceAlerts,
            updatePriceAlerts = updatePriceAlerts,
        )
    }
}
