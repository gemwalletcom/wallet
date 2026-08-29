package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.application.perpetual.cases.GetPerpetualAccountMode
import com.gemwallet.android.application.perpetual.cases.SyncPerpetualPositions
import com.gemwallet.android.application.perpetual.cases.SyncPerpetuals
import com.gemwallet.android.cases.nodes.GetNodeUrlCase
import com.gemwallet.android.data.repositories.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.repositories.perpetual.ObservePerpetualWallet
import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStore
import com.gemwallet.android.data.repositories.stream.ExponentialReconnection
import com.gemwallet.android.data.repositories.stream.WebSocketConnection
import com.gemwallet.android.data.repositories.stream.WebSocketRequest
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPerpetualStreamService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemPriceService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import javax.inject.Singleton
import com.gemwallet.android.data.repositories.gemstone.GemstonePerpetualStreamConnection

@InstallIn(SingletonComponent::class)
@Module
object PerpetualModule {

    @Provides
    @Singleton
    fun provideGemstonePerpetualStore(
        perpetualDao: PerpetualDao,
        searchDao: SearchDao,
        perpetualPositionDao: PerpetualPositionDao,
        balancesDao: BalancesDao,
        transactionRunner: StoreTransactionRunner,
    ): GemstonePerpetualStore = GemstonePerpetualStore(perpetualDao, searchDao, perpetualPositionDao, balancesDao, transactionRunner)

    @Provides
    @Singleton
    fun provideGemPerpetualService(
        gateway: GemGateway,
        priceService: GemPriceService,
        perpetualStore: GemstonePerpetualStore,
        assetStore: GemAssetStore,
        preferencesService: GemPreferencesService,
        balanceService: GemBalanceService,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemPerpetualService =
        GemPerpetualService(gateway, priceService, perpetualStore, assetStore, preferencesService, balanceService, walletPreferencesService)

    @Provides
    @Singleton
    fun provideHyperliquidObserverService(
        observePerpetualWallet: ObservePerpetualWallet,
        syncPerpetuals: SyncPerpetuals,
        syncPerpetualPositions: SyncPerpetualPositions,
        getPerpetualAccountMode: GetPerpetualAccountMode,
        perpetualService: GemPerpetualService,
        getNodeUrlCase: GetNodeUrlCase,
        okHttpClient: OkHttpClient,
    ): HyperliquidObserverService {
        val connection = WebSocketConnection(
            client = okHttpClient,
            requestProvider = {
                WebSocketRequest(url = getNodeUrlCase.getWebSocketNodeUrl(Chain.HyperCore))
            },
            reconnection = ExponentialReconnection(maxDelay = 30.0),
        )
        return HyperliquidObserverService(
            observePerpetualWallet = observePerpetualWallet,
            syncPerpetuals = syncPerpetuals,
            syncPerpetualPositions = syncPerpetualPositions,
            getPerpetualAccountMode = getPerpetualAccountMode,
            streamService = GemPerpetualStreamService(perpetualService, GemstonePerpetualStreamConnection(connection)),
            connection = connection,
        )
    }

    @Provides
    @Singleton
    fun providePerpetualObserver(service: HyperliquidObserverService): PerpetualObserver = service
}
