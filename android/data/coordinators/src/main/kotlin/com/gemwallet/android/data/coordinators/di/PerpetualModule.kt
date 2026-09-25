package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPosition
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositionByAsset
import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.GetPerpetuals
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionByAssetImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualPositionsImpl
import com.gemwallet.android.data.coordinators.perpetuals.GetPerpetualsImpl
import com.gemwallet.android.data.coordinators.perpetuals.PerpetualBalanceCoordinator
import com.gemwallet.android.data.services.store.database.PricesDao
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PerpetualModule {
    @Provides
    @Singleton
    fun provideGetPerpetualPositions(getSession: GetSession, perpetualStore: GemstonePerpetualStore): GetPerpetualPositions = GetPerpetualPositionsImpl(
        getSession = getSession,
        perpetualStore = perpetualStore,
    )

    @Provides
    @Singleton
    fun provideGetPerpetualPositionByAsset(perpetualStore: GemstonePerpetualStore): GetPerpetualPositionByAsset = GetPerpetualPositionByAssetImpl(perpetualStore)

    @Provides
    @Singleton
    fun provideGetPerpetualPosition(perpetualStore: GemstonePerpetualStore): GetPerpetualPosition = GetPerpetualPositionImpl(
        perpetualStore = perpetualStore,
    )

    @Provides
    @Singleton
    fun provideGetPerpetuals(perpetualStore: GemstonePerpetualStore): GetPerpetuals = GetPerpetualsImpl(
        perpetualStore = perpetualStore,
    )

    @Provides
    @Singleton
    fun provideGetPerpetual(perpetualStore: GemstonePerpetualStore): GetPerpetual = GetPerpetualImpl(
        perpetualStore = perpetualStore,
    )

    @Provides
    @Singleton
    fun provideGetPerpetualBalance(perpetualStore: GemstonePerpetualStore, getSession: GetSession, pricesDao: PricesDao): GetPerpetualBalance = PerpetualBalanceCoordinator(
        perpetualStore = perpetualStore,
        getSession = getSession,
        pricesDao = pricesDao,
    )
}
