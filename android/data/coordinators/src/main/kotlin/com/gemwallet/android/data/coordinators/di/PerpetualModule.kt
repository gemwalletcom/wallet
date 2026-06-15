package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.perpetual.coordinators.BuildPerpetualParams
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalances
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualChartData
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualChartPeriod
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetuals
import com.gemwallet.android.application.perpetual.coordinators.SetPerpetualChartPeriod
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
import com.gemwallet.android.application.perpetual.coordinators.TogglePerpetualPin
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.data.coordinators.perpetuals.BuildPerpetualParamsImpl
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
import com.gemwallet.android.data.coordinators.perpetuals.TogglePerpetualPinImpl
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.PricesDao
import com.wallet.core.primitives.Chain
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PerpetualModule {
    @Provides
    @Singleton
    fun provideSyncPerpetuals(
        perpetualService: PerpetualService,
        perpetualRepository: PerpetualRepository,
        pricesDao: PricesDao,
        sessionRepository: SessionRepository,
    ): SyncPerpetuals {
        return SyncPerpetualsImpl(
            perpetualService = perpetualService,
            perpetualRepository = perpetualRepository,
            pricesDao = pricesDao,
            sessionRepository = sessionRepository,
            chains = listOf(Chain.HyperCore)
        )
    }

    @Provides
    @Singleton
    fun provideSyncPerpetualPositions(
        sessionRepository: SessionRepository,
        perpetualService: PerpetualService,
        perpetualRepository: PerpetualRepository,
    ): SyncPerpetualPositions {
        return SyncPerpetualPositionsImpl(
            sessionRepository = sessionRepository,
            perpetualService = perpetualService,
            perpetualRepository = perpetualRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualPositions(
        sessionRepository: SessionRepository,
        perpetualRepository: PerpetualRepository,
    ): GetPerpetualPositions {
        return GetPerpetualPositionsImpl(
            sessionRepository = sessionRepository,
            perpetualRepository = perpetualRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualPosition(
        perpetualRepository: PerpetualRepository,
    ): GetPerpetualPosition {
        return GetPerpetualPositionImpl(
            perpetualRepository = perpetualRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetuals(
        perpetualRepository: PerpetualRepository,
    ): GetPerpetuals {
        return GetPerpetualsImpl(
            perpetualRepository = perpetualRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetual(
        perpetualRepository: PerpetualRepository,
    ): GetPerpetual {
        return GetPerpetualImpl(
            perpetualRepository = perpetualRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualBalances(
        sessionRepository: SessionRepository,
        perpetualRepository: PerpetualRepository,
    ): GetPerpetualBalances {
        return GetPerpetualBalancesImpl(
            sessionRepository = sessionRepository,
            perpetualRepository = perpetualRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualBalance(
        perpetualRepository: PerpetualRepository,
        sessionRepository: SessionRepository,
    ): GetPerpetualBalance {
        return GetPerpetualBalanceImpl(
            perpetualRepository = perpetualRepository,
            sessionRepository = sessionRepository,
        )
    }

    @Provides
    @Singleton
    fun provideTogglePerpetualPin(
        perpetualRepository: PerpetualRepository,
    ): TogglePerpetualPin {
        return TogglePerpetualPinImpl(
            perpetualRepository = perpetualRepository,
        )
    }

    @Provides
    @Singleton
    fun provideGetPerpetualChartData(
        perpetualService: PerpetualService,
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
        perpetualRepository: PerpetualRepository,
        sessionRepository: SessionRepository,
    ): BuildPerpetualParams {
        return BuildPerpetualParamsImpl(
            perpetualRepository = perpetualRepository,
            sessionRepository = sessionRepository,
        )
    }
}