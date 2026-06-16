package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.perpetual.coordinators.PerpetualObserver
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.cases.nodes.GetCurrentNodeCase
import com.gemwallet.android.cases.nodes.GetNodesCase
import com.gemwallet.android.cases.nodes.SetCurrentNodeCase
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.perpetual.HyperliquidEventHandler
import com.gemwallet.android.data.repositories.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepositoryImpl
import com.gemwallet.android.data.repositories.perpetual.toWebSocketUrl
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.ExponentialReconnection
import com.gemwallet.android.data.repositories.stream.WebSocketConnection
import com.gemwallet.android.data.repositories.stream.WebSocketRequest
import com.gemwallet.android.data.services.gemapi.http.getNodeUrl
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.wallet.core.primitives.Chain

import com.gemwallet.android.data.service.store.database.SearchDao
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
    fun providePerpetualRepository(
        perpetualDao: PerpetualDao,
        perpetualPositionDao: PerpetualPositionDao,
        assetsDao: AssetsDao,
        balancesDao: BalancesDao,
        searchDao: SearchDao,
    ): PerpetualRepository {
        return PerpetualRepositoryImpl(
            perpetualDao = perpetualDao,
            perpetualPositionDao = perpetualPositionDao,
            assetsDao = assetsDao,
            balancesDao = balancesDao,
            searchDao = searchDao,
        )
    }

    @Provides
    @Singleton
    fun provideHyperliquidEventHandler(
        perpetualRepository: PerpetualRepository,
    ): HyperliquidEventHandler = HyperliquidEventHandler(
        perpetualRepository = perpetualRepository,
    )

    @Provides
    @Singleton
    fun provideHyperliquidObserverService(
        sessionRepository: SessionRepository,
        userConfig: UserConfig,
        syncPerpetualPositions: SyncPerpetualPositions,
        eventHandler: HyperliquidEventHandler,
        getNodesCase: GetNodesCase,
        getCurrentNodeCase: GetCurrentNodeCase,
        setCurrentNodeCase: SetCurrentNodeCase,
    ): HyperliquidObserverService = HyperliquidObserverService(
        sessionRepository = sessionRepository,
        userConfig = userConfig,
        syncPerpetualPositions = syncPerpetualPositions,
        eventHandler = eventHandler,
        connection = WebSocketConnection(
            requestProvider = {
                val url = Chain.HyperCore.getNodeUrl(getNodesCase, getCurrentNodeCase, setCurrentNodeCase)
                    ?: error("No node url for ${Chain.HyperCore.string}")
                WebSocketRequest(url = url.toWebSocketUrl())
            },
            reconnection = ExponentialReconnection(maxDelay = 30.0),
        ),
    )

    @Provides
    @Singleton
    fun providePerpetualObserver(service: HyperliquidObserverService): PerpetualObserver = service
}
