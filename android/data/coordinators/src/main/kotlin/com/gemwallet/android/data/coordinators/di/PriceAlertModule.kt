package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.pricealerts.coordinators.ExcludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.IncludePriceAlert
import com.gemwallet.android.application.pricealerts.coordinators.PriceAlertsStateCoordinator
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.cases.device.SyncDeviceInfo
import com.gemwallet.android.data.coordinators.pricealerts.ExcludePriceAlertImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetAssetPriceAlertStateImpl
import com.gemwallet.android.data.coordinators.pricealerts.GetPriceAlertsImpl
import com.gemwallet.android.data.coordinators.pricealerts.IncludePriceAlertImpl
import com.gemwallet.android.data.coordinators.pricealerts.PriceAlertsStateCoordinatorImpl
import com.gemwallet.android.data.coordinators.pricealerts.UpdatePriceAlertsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PriceAlertModule {
    @Provides
    @Singleton
    fun provideAddPriceAlerts(
        gemDeviceApiClient: GemDeviceApiClient,
        priceAlertRepository: PriceAlertRepository,
        sessionRepository: SessionRepository,
        syncDeviceInfo: SyncDeviceInfo,
    ): IncludePriceAlert {
        return IncludePriceAlertImpl(
            gemDeviceApiClient = gemDeviceApiClient,
            priceAlertRepository = priceAlertRepository,
            sessionRepository = sessionRepository,
            syncDeviceInfo = syncDeviceInfo,
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
    fun providePriceAlertsStateCoordinator(
        priceAlertRepository: PriceAlertRepository,
        includePriceAlert: IncludePriceAlert,
        excludePriceAlert: ExcludePriceAlert,
        syncDeviceInfo: SyncDeviceInfo,
    ): PriceAlertsStateCoordinator {
        return PriceAlertsStateCoordinatorImpl(
            priceAlertRepository = priceAlertRepository,
            includePriceAlert = includePriceAlert,
            excludePriceAlert = excludePriceAlert,
            syncDeviceInfo = syncDeviceInfo,
        )
    }

    @Provides
    @Singleton
    fun providePriceAlertExclude(
        gemDeviceApiClient: GemDeviceApiClient,
        sessionRepository: SessionRepository,
        priceAlertRepository: PriceAlertRepository,
    ): ExcludePriceAlert {
        return ExcludePriceAlertImpl(
            gemDeviceApiClient = gemDeviceApiClient,
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
        gemDeviceApiClient: GemDeviceApiClient,
        priceAlertRepository: PriceAlertRepository,
    ): UpdatePriceAlerts {
        return UpdatePriceAlertsImpl(
            gemDeviceApiClient = gemDeviceApiClient,
            priceAlertRepository = priceAlertRepository,
        )
    }
}
