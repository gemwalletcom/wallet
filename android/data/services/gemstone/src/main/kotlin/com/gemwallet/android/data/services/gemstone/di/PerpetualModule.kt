package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.application.perpetual.cases.PerpetualObserver
import com.gemwallet.android.data.services.gemstone.perpetual.GemstonePerpetualStreamConnection
import com.gemwallet.android.data.services.gemstone.perpetual.HyperliquidObserverService
import com.gemwallet.android.data.services.gemstone.perpetual.ObservePerpetualWallet
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnection
import com.gemwallet.android.data.services.gemstone.stream.WebSocketRequest
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.BalancesDao
import com.gemwallet.android.data.services.store.database.PerpetualDao
import com.gemwallet.android.data.services.store.database.PerpetualPositionDao
import com.gemwallet.android.data.services.store.database.StoreTransactionRunner
import com.wallet.core.primitives.Chain
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemNodeServiceInterface
import uniffi.gemstone.GemPerpetualDetailsService
import uniffi.gemstone.GemPerpetualDetailsServiceInterface
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPerpetualServiceInterface
import uniffi.gemstone.GemPerpetualStreamService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemRecentActivityService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object PerpetualModule {

    @Provides
    @Singleton
    fun provideGemstonePerpetualStore(perpetualDao: PerpetualDao, perpetualPositionDao: PerpetualPositionDao, balancesDao: BalancesDao, transactionRunner: StoreTransactionRunner): GemstonePerpetualStore =
        GemstonePerpetualStore(perpetualDao, perpetualPositionDao, balancesDao, transactionRunner)

    @Provides
    @Singleton
    fun provideGemPerpetualService(
        gateway: GemGateway,
        priceService: GemPriceService,
        perpetualStore: GemstonePerpetualStore,
        assetsService: GemAssetsService,
        preferencesService: GemPreferencesService,
        balanceService: GemBalanceService,
        walletPreferencesService: GemWalletPreferencesService,
        walletSessionService: GemWalletSessionService,
        recentActivityService: GemRecentActivityService,
    ): GemPerpetualService = GemPerpetualService(
        gateway,
        priceService,
        perpetualStore,
        assetsService,
        preferencesService,
        balanceService,
        walletPreferencesService,
        walletSessionService,
        recentActivityService,
    )

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
