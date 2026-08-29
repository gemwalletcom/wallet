package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.perpetual.cases.BuildPerpetualParams
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualAccountMode
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalances
import com.gemwallet.android.application.perpetual.cases.GetPerpetualChartData
import com.gemwallet.android.application.perpetual.cases.GetPerpetualChartPeriod
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.perpetual.cases.SetPerpetualChartPeriod
import com.gemwallet.android.application.perpetual.cases.SyncPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.SyncPerpetuals
import com.gemwallet.android.application.perpetual.cases.SetPerpetualPinned
import com.gemwallet.android.data.coordinators.perpetuals.BuildPerpetualParamsImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualAccountModeImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualBalanceImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualBalancesImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualChartDataImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualChartPeriodImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionsImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualsImpl
import com.gemwallet.android.data.coordinators.perpetuals.SetPerpetualChartPeriodImpl
import com.gemwallet.android.data.coordinators.perpetuals.SyncPerpetualPositionsImpl
import com.gemwallet.android.data.coordinators.perpetuals.SyncPerpetualsImpl
import com.gemwallet.android.data.coordinators.perpetuals.SetPerpetualPinnedImpl
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStore
import com.gemwallet.android.data.repositories.session.SessionRepository
import uniffi.gemstone.GemPerpetualService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionByAssetImpl
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositionByAsset

@InstallIn(SingletonComponent::class)
@Module
object PerpetualModule {
    @Provides
    @Singleton
    fun provideSyncPerpetuals(
        perpetualService: GemPerpetualService,
    ): SyncPerpetuals {
        return SyncPerpetualsImpl(perpetualService = perpetualService)
    }

    @Provides
    @Singleton
    fun provideGetPerpetualAccountMode(
        perpetualService: GemPerpetualService,
    ): GetPerpetualAccountMode {
        return GetPerpetualAccountModeImpl(perpetualService)
    }

    @Provides
    @Singleton
    fun provideSyncPerpetualPositions(
        sessionRepository: SessionRepository,
        perpetualService: GemPerpetualService,
    ): SyncPerpetualPositions {
        return SyncPerpetualPositionsImpl(
            sessionRepository = sessionRepository,
            perpetualService = perpetualService,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualPositions(
        sessionRepository: SessionRepository,
        perpetualStore: GemstonePerpetualStore,
    ): GetPerpetualPositions {
        return GetPerpetualPositionsImpl(
            sessionRepository = sessionRepository,
            perpetualStore = perpetualStore,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualPositionByAsset(
        perpetualStore: GemstonePerpetualStore,
    ): GetPerpetualPositionByAsset = GetPerpetualPositionByAssetImpl(perpetualStore)

    @Provides
    @Singleton
    fun provideGetPerpetualPosition(
        perpetualStore: GemstonePerpetualStore,
    ): GetPerpetualPosition {
        return GetPerpetualPositionImpl(
            perpetualStore = perpetualStore,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetuals(
        perpetualStore: GemstonePerpetualStore,
    ): GetPerpetuals {
        return GetPerpetualsImpl(
            perpetualStore = perpetualStore,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetual(
        perpetualStore: GemstonePerpetualStore,
    ): GetPerpetual {
        return GetPerpetualImpl(
            perpetualStore = perpetualStore,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualBalances(
        sessionRepository: SessionRepository,
        perpetualStore: GemstonePerpetualStore,
    ): GetPerpetualBalances {
        return GetPerpetualBalancesImpl(
            sessionRepository = sessionRepository,
            perpetualStore = perpetualStore,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualBalance(
        perpetualStore: GemstonePerpetualStore,
        sessionRepository: SessionRepository,
    ): GetPerpetualBalance {
        return GetPerpetualBalanceImpl(
            perpetualStore = perpetualStore,
            sessionRepository = sessionRepository,
        )
    }

    @Provides
    @Singleton
    fun provideSetPerpetualPinned(perpetualService: GemPerpetualService): SetPerpetualPinned {
        return SetPerpetualPinnedImpl(perpetualService)
    }

    @Provides
    @Singleton
    fun provideGetPerpetualChartData(
        perpetualService: GemPerpetualService,
    ): GetPerpetualChartData {
        return GetPerpetualChartDataImpl(
            perpetualService = perpetualService,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualChartPeriod(
        userConfig: UserConfig,
    ): GetPerpetualChartPeriod {
        return GetPerpetualChartPeriodImpl(
            userConfig = userConfig,
        )
    }

    @Provides
    @Singleton
    fun provideSetPerpetualChartPeriod(
        userConfig: UserConfig,
    ): SetPerpetualChartPeriod {
        return SetPerpetualChartPeriodImpl(
            userConfig = userConfig,
        )
    }

    @Provides
    @Singleton
    fun provideBuildPerpetualParams(
        perpetualStore: GemstonePerpetualStore,
        sessionRepository: SessionRepository,
    ): BuildPerpetualParams {
        return BuildPerpetualParamsImpl(
            perpetualStore = perpetualStore,
            sessionRepository = sessionRepository,
        )
    }
}