package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.perpetual.cases.BuildPerpetualParams
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.data.coordinators.perpetuals.BuildPerpetualParamsImpl
import com.gemwallet.android.data.services.gemstone.perpetual.ObservePerpetualWallet
import com.gemwallet.android.data.coordinators.perpetuals.PerpetualBalanceCoordinator
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionsImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualsImpl
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.application.session.cases.GetSession
import uniffi.gemstone.GemWalletPreferencesService
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
    fun provideGetPerpetualPositions(
        getSession: GetSession,
        perpetualStore: GemstonePerpetualStore,
    ): GetPerpetualPositions {
        return GetPerpetualPositionsImpl(
            getSession = getSession,
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
    fun provideGetPerpetualBalance(
        perpetualStore: GemstonePerpetualStore,
        getSession: GetSession,
        observePerpetualWallet: ObservePerpetualWallet,
        walletPreferencesService: GemWalletPreferencesService,
    ): GetPerpetualBalance {
        return PerpetualBalanceCoordinator(
            perpetualStore = perpetualStore,
            getSession = getSession,
            observePerpetualWallet = observePerpetualWallet,
            walletPreferencesService = walletPreferencesService,
        )
    }

    @Provides
    @Singleton
    fun provideBuildPerpetualParams(
        perpetualStore: GemstonePerpetualStore,
        getSession: GetSession,
        service: GemPerpetualDetailsServiceInterface,
    ): BuildPerpetualParams {
        return BuildPerpetualParamsImpl(
            perpetualStore = perpetualStore,
            getSession = getSession,
            service = service,
        )
    }
}