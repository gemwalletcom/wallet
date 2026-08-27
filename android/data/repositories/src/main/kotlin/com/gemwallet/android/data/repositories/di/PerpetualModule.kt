package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.perpetual.coordinators.PerpetualObserver
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualAccountMode
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions
import com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals
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
import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStore
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import javax.inject.Singleton
import uniffi.gemstone.Hyperliquid
import uniffi.gemstone.HyperliquidSubscriptions

@InstallIn(SingletonComponent::class)
@Module
object PerpetualModule {

    @Provides
    @Singleton
    fun provideGemstonePerpetualStore(
        perpetualDao: PerpetualDao,
        perpetualPositionDao: PerpetualPositionDao,
        assetsDao: AssetsDao,
        balancesDao: BalancesDao,
    ): GemstonePerpetualStore = GemstonePerpetualStore(perpetualDao, perpetualPositionDao, assetsDao, balancesDao)

    @Provides
    @Singleton
    fun provideGemPerpetualService(
        gateway: GemGateway,
        priceService: GemPriceService,
        perpetualStore: GemstonePerpetualStore,
        preferencesService: GemPreferencesService,
    ): GemPerpetualService = GemPerpetualService(gateway, priceService, perpetualStore, preferencesService)

    @Provides
    @Singleton
    fun providePerpetualRepository(
        perpetualDao: PerpetualDao,
        perpetualPositionDao: PerpetualPositionDao,
        balancesDao: BalancesDao,
        searchDao: SearchDao,
        perpetualStore: GemstonePerpetualStore,
    ): PerpetualRepository {
        return PerpetualRepositoryImpl(
            perpetualDao = perpetualDao,
            perpetualPositionDao = perpetualPositionDao,
            balancesDao = balancesDao,
            searchDao = searchDao,
            perpetualStore = perpetualStore,
        )
    }

    @Provides
    @Singleton
    fun provideHyperliquid(): Hyperliquid = Hyperliquid()

    @Provides
    @Singleton
    fun provideHyperliquidEventHandler(
        perpetualService: GemPerpetualService,
        hyperliquid: Hyperliquid,
    ): HyperliquidEventHandler = HyperliquidEventHandler(
        perpetualService = perpetualService,
        hyperliquid = hyperliquid,
    )

    @Provides
    @Singleton
    fun provideHyperliquidSubscriptionService(): HyperliquidSubscriptionService =
        HyperliquidSubscriptionService(HyperliquidSubscriptions())

    @Provides
    @Singleton
    fun provideHyperliquidObserverService(
        observePerpetualWallet: ObservePerpetualWallet,
        syncPerpetuals: SyncPerpetuals,
        syncPerpetualPositions: SyncPerpetualPositions,
        getPerpetualAccountMode: GetPerpetualAccountMode,
        eventHandler: HyperliquidEventHandler,
        subscriptionService: HyperliquidSubscriptionService,
        getNodeUrlCase: GetNodeUrlCase,
        okHttpClient: OkHttpClient,
    ): HyperliquidObserverService = HyperliquidObserverService(
        observePerpetualWallet = observePerpetualWallet,
        syncPerpetuals = syncPerpetuals,
        syncPerpetualPositions = syncPerpetualPositions,
        getPerpetualAccountMode = getPerpetualAccountMode,
        eventHandler = eventHandler,
        subscriptionService = subscriptionService,
        connection = WebSocketConnection(
            client = okHttpClient,
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
