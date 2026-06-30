package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.perpetual.coordinators.PerpetualObserver
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.data.repositories.perpetual.HyperliquidEventHandler
import com.gemwallet.android.data.repositories.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.repositories.perpetual.HyperliquidSubscriptionService
import com.gemwallet.android.data.repositories.perpetual.ObservePerpetualWallet
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepositoryImpl
import com.gemwallet.android.data.repositories.perpetual.toWebSocketUrl
import com.gemwallet.android.data.repositories.stream.ExponentialReconnection
import com.gemwallet.android.data.repositories.stream.WebSocketConnection
import com.gemwallet.android.data.repositories.stream.WebSocketRequest
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.wallet.core.primitives.Chain
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import uniffi.gemstone.Hyperliquid

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
    fun provideHyperliquid(): Hyperliquid = Hyperliquid()

    @Provides
    @Singleton
    fun provideHyperliquidEventHandler(
        perpetualRepository: PerpetualRepository,
        hyperliquid: Hyperliquid,
    ): HyperliquidEventHandler = HyperliquidEventHandler(
        perpetualRepository = perpetualRepository,
        hyperliquid = hyperliquid,
    )

    @Provides
    @Singleton
    fun provideHyperliquidSubscriptionService(
        hyperliquid: Hyperliquid,
    ): HyperliquidSubscriptionService =
        HyperliquidSubscriptionService(hyperliquid::websocketRequest)

    @Provides
    @Singleton
    fun provideHyperliquidObserverService(
        observePerpetualWallet: ObservePerpetualWallet,
        syncPerpetualPositions: SyncPerpetualPositions,
        eventHandler: HyperliquidEventHandler,
        subscriptionService: HyperliquidSubscriptionService,
        getNodeUrlCase: GetNodeUrlCase,
    ): HyperliquidObserverService = HyperliquidObserverService(
        observePerpetualWallet = observePerpetualWallet,
        syncPerpetualPositions = syncPerpetualPositions,
        eventHandler = eventHandler,
        subscriptionService = subscriptionService,
        connection = WebSocketConnection(
            requestProvider = {
                val url = getNodeUrlCase.getNodeUrl(Chain.HyperCore)
                WebSocketRequest(url = url.toWebSocketUrl())
            },
            reconnection = ExponentialReconnection(maxDelay = 30.0),
        ),
    )

    @Provides
    @Singleton
    fun providePerpetualObserver(service: HyperliquidObserverService): PerpetualObserver = service
}
