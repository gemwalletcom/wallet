package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import uniffi.gemstone.GemNodeServiceInterface
import com.gemwallet.android.data.services.gemstone.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.services.gemstone.perpetual.ObservePerpetualWallet
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnection
import com.gemwallet.android.data.services.gemstone.stream.WebSocketRequest
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PerpetualDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.PerpetualPositionDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemPerpetualDetailsService
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualStreamService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemPriceService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import javax.inject.Singleton
import com.gemwallet.android.data.services.gemstone.perpetual.GemstonePerpetualStreamConnection

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
        walletSessionService: GemWalletSessionService,
    ): GemPerpetualService =
        GemPerpetualService(gateway, priceService, perpetualStore, assetStore, preferencesService, balanceService, walletPreferencesService, walletSessionService)

    @Provides
    @Singleton
    fun provideGemPerpetualServiceInterface(service: GemPerpetualService): GemPerpetualServiceInterface = service

    @Provides
    fun provideGemPerpetualDetailsService(
        perpetualService: GemPerpetualService,
        transactionsService: GemTransactionsService,
        preferencesService: GemPreferencesService,
        walletSessionService: GemWalletSessionService,
    ): GemPerpetualDetailsServiceInterface = GemPerpetualDetailsService(perpetualService, transactionsService, preferencesService, walletSessionService)

    @Provides
    @Singleton
    fun provideHyperliquidObserverService(
        observePerpetualWallet: ObservePerpetualWallet,
        perpetualService: GemPerpetualService,
        nodeService: GemNodeServiceInterface,
        okHttpClient: OkHttpClient,
        connectionService: GemConnectionService,
    ): HyperliquidObserverService {
        val connection = WebSocketConnection(
            client = okHttpClient,
            requestProvider = {
                WebSocketRequest(url = nodeService.websocketNodeUrl(Chain.HyperCore.string))
            },
            connectionService = connectionService,
        )
        return HyperliquidObserverService(
            observePerpetualWallet = observePerpetualWallet,
            perpetualService = perpetualService,
            streamService = GemPerpetualStreamService(perpetualService, GemstonePerpetualStreamConnection(connection)),
            connection = connection,
        )
    }

    @Provides
    @Singleton
    fun providePerpetualObserver(service: HyperliquidObserverService): PerpetualObserver = service
}
