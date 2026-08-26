package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.coordinators.HasAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.SetAssetPriceAlertEnabled
import com.gemwallet.android.application.pricealerts.coordinators.SetPriceAlertsEnabled
import com.gemwallet.android.application.pricealerts.coordinators.SyncAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.data.coordinators.pricealerts.ExcludePriceAlertImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetAssetPriceAlertStateImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetPriceAlertsEnabledImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.HasAssetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.IncludePriceAlertImpl
import com.gemwallet.android.data.coordinators.pricealerts.SetAssetPriceAlertEnabledImpl
import com.gemwallet.android.data.coordinators.pricealerts.SetPriceAlertsEnabledImpl
import com.gemwallet.android.data.coordinators.pricealerts.SyncAssetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.UpdatePriceAlertsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPriceAlertService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PriceAlertModule {
    @Provides
    @Singleton
    fun provideAddPriceAlerts(
        priceAlertService: GemPriceAlertService,
        priceAlertRepository: PriceAlertRepository,
        sessionRepository: SessionRepository,
        syncDevice: SyncDevice,
    ): IncludePriceAlert {
        return IncludePriceAlertImpl(
            priceAlertService = priceAlertService,
            priceAlertRepository = priceAlertRepository,
            sessionRepository = sessionRepository,
            syncDevice = syncDevice,
        )
    }

    @Provides
    @Singleton
    fun provideGetPriceAlerts(
        priceAlertRepository: PriceAlertRepository,
        assetsRepository: AssetsRepository,
    ): GetPriceAlerts {
        return GetPriceAlertsImpl(
            priceAlertRepository = priceAlertRepository,
            assetsRepository = assetsRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPriceAlertsEnabled(
        priceAlertRepository: PriceAlertRepository,
    ): GetPriceAlertsEnabled {
        return GetPriceAlertsEnabledImpl(
            priceAlertRepository = priceAlertRepository,
        )
    }

    @Provides
    @Singleton
    fun provideSetPriceAlertsEnabled(
        priceAlertRepository: PriceAlertRepository,
        syncDevice: SyncDevice,
    ): SetPriceAlertsEnabled {
        return SetPriceAlertsEnabledImpl(
            priceAlertRepository = priceAlertRepository,
            syncDevice = syncDevice,
        )
    }

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
        sessionRepository: SessionRepository,
        priceAlertRepository: PriceAlertRepository,
    ): ExcludePriceAlert {
        return ExcludePriceAlertImpl(
            priceAlertService = priceAlertService,
            sessionRepository = sessionRepository,
            priceAlertRepository = priceAlertRepository,
        )
    }

    @Provides
    @Singleton
    fun provideAssetPriceAlertState(
        priceAlertRepository: PriceAlertRepository,
    ): GetAssetPriceAlertState {
        return GetAssetPriceAlertStateImpl(
            priceAlertRepository = priceAlertRepository,
        )
    }

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
        priceAlertRepository: PriceAlertRepository,
    ): HasAssetPriceAlerts = HasAssetPriceAlertsImpl(priceAlertRepository)

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
