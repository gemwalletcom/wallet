package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.services.gemstone.connection.ConnectionComponentHealth
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBalanceStore
import com.gemwallet.android.data.services.gemstone.stores.GemstonePortfolioStore
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.data.services.gemstone.stream.GemstoneStreamConnection
import com.gemwallet.android.data.services.gemstone.stream.StreamObserverService
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnectable
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnection
import com.gemwallet.android.data.services.gemstone.stream.WebSocketRequest
import com.gemwallet.android.math.fromHex
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import uniffi.gemstone.GemAddAssetService
import uniffi.gemstone.GemAddAssetServiceInterface
import uniffi.gemstone.GemAddressDetailsService
import uniffi.gemstone.GemAddressDetailsServiceInterface
import uniffi.gemstone.GemApiClient
import uniffi.gemstone.GemAssetDetailsService
import uniffi.gemstone.GemAssetDetailsServiceInterface
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemAssetsServiceInterface
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemBalanceStore
import uniffi.gemstone.GemBannerService
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemDeviceKeyService
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemFiatService
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemNotificationStore
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemPortfolioStore
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemPriceAlertStore
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemReceiveService
import uniffi.gemstone.GemReceiveServiceInterface
import uniffi.gemstone.GemRecentActivityService
import uniffi.gemstone.GemStreamService
import uniffi.gemstone.GemStreamServiceInterface
import uniffi.gemstone.GemStreamSubscriptionService
import uniffi.gemstone.GemSupportStore
import uniffi.gemstone.GemSwapService
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object AssetsModule {

    @Provides
    @Singleton
    fun provideGemBalanceStore(assetsDao: AssetsDao, balancesDao: BalancesDao, transactionRunner: StoreTransactionRunner): GemBalanceStore = GemstoneBalanceStore(balancesDao, assetsDao, transactionRunner)

    @Provides
    @Singleton
    fun provideGemBalanceService(
        gateway: GemGateway,
        walletStore: GemstoneWalletStore,
        assetStore: GemAssetStore,
        balanceStore: GemBalanceStore,
        assetsService: GemAssetsService,
        streamSubscriptionService: GemStreamSubscriptionService,
    ): GemBalanceService = GemBalanceService(
        gateway,
        walletStore,
        assetStore,
        balanceStore,
        assetsService,
        streamSubscriptionService,
    )

    @Provides
    @Singleton
    fun provideGemStreamService(
        priceService: GemPriceService,
        priceAlertService: GemPriceAlertService,
        balanceService: GemBalanceService,
        transactionsService: GemTransactionsService,
        nftService: GemNftService,
        perpetualService: GemPerpetualService,
        fiatService: GemFiatService,
        notificationStore: GemNotificationStore,
        supportStore: GemSupportStore,
        subscriptions: GemStreamSubscriptionService,
        preferences: GemPreferencesService,
        session: GemWalletSessionService,
        device: GemDeviceService,
    ): GemStreamService = GemStreamService(
        priceService,
        priceAlertService,
        balanceService,
        transactionsService,
        nftService,
        perpetualService,
        fiatService,
        notificationStore,
        supportStore,
        subscriptions,
        preferences,
        session,
        device,
    )

    @Provides
    @Singleton
    fun provideGemStreamServiceInterface(service: GemStreamService): GemStreamServiceInterface = service

    @Provides
    @Singleton
    fun provideStreamConnection(deviceKeyService: GemDeviceKeyService, okHttpClient: OkHttpClient, connectionService: GemConnectionService): WebSocketConnectable = WebSocketConnection(
        client = okHttpClient,
        requestProvider = {
            withContext(Dispatchers.IO) {
                val stream = deviceKeyService.deviceStreamRequest()
                WebSocketRequest(url = stream.url, headers = mapOf("Authorization" to stream.authorization))
            }
        },
        connectionService = connectionService,
    )

    @Provides
    @Singleton
    fun provideStreamSubscriptionService(balanceStore: GemBalanceStore, priceAlertStore: GemPriceAlertStore, connection: WebSocketConnectable): GemStreamSubscriptionService = GemStreamSubscriptionService(
        balances = balanceStore,
        alerts = priceAlertStore,
        connection = GemstoneStreamConnection(connection),
    )

    @Provides
    @Singleton
    fun provideStreamObserverService(getSession: GetSession, streamService: GemStreamServiceInterface, connection: WebSocketConnectable, streamHealth: ConnectionComponentHealth): StreamObserverService = StreamObserverService(
        getSession = getSession,
        service = streamService,
        connection = connection,
        health = streamHealth,
    )

    @Provides
    @Singleton
    fun provideGemstoneAssetStore(assetsDao: AssetsDao): GemstoneAssetStore = GemstoneAssetStore(assetsDao)

    @Provides
    @Singleton
    fun provideGemAssetStore(store: GemstoneAssetStore): GemAssetStore = store

    @Provides
    @Singleton
    fun provideGemPortfolioStore(assetsDao: AssetsDao): GemPortfolioStore = GemstonePortfolioStore(assetsDao)

    @Provides
    @Singleton
    fun provideGemAssetDetailsService(
        assetsService: GemAssetsService,
        balanceService: GemBalanceService,
        transactionsService: GemTransactionsService,
        bannerService: GemBannerService,
        swapService: GemSwapServiceInterface,
        explorerService: GemExplorerService,
        priceAlertService: GemPriceAlertService,
        streamSubscriptionService: GemStreamSubscriptionService,
        deeplinkService: GemDeeplinkService,
        walletSessionService: GemWalletSessionService,
    ): GemAssetDetailsService = GemAssetDetailsService(
        assetsService,
        balanceService,
        transactionsService,
        bannerService,
        swapService as GemSwapService,
        explorerService,
        priceAlertService,
        streamSubscriptionService,
        deeplinkService,
        walletSessionService,
    )

    @Provides
    @Singleton
    fun provideGemAssetsService(apiClient: GemApiClient, gateway: GemGateway, assetStore: GemAssetStore, priceService: GemPriceService, preferencesService: GemPreferencesService, session: GemWalletSessionService): GemAssetsService =
        GemAssetsService(apiClient, gateway, assetStore, priceService, preferencesService, session)

    @Provides
    fun provideGemReceiveService(balanceService: GemBalanceService, assetsService: GemAssetsService, recentActivityService: GemRecentActivityService): GemReceiveServiceInterface =
        GemReceiveService(balanceService, assetsService, recentActivityService)

    @Provides
    fun provideGemAddressDetailsService(gateway: GemGateway, explorerService: GemExplorerService, nameService: GemNameService): GemAddressDetailsServiceInterface = GemAddressDetailsService(gateway, explorerService, nameService)

    @Provides
    fun provideGemAddAssetService(assetsService: GemAssetsService, balanceService: GemBalanceService, explorerService: GemExplorerService): GemAddAssetServiceInterface = GemAddAssetService(assetsService, balanceService, explorerService)

    @Provides
    @Singleton
    fun provideGemstonePriceStore(pricesDao: PricesDao, assetsDao: AssetsDao): GemstonePriceStore = GemstonePriceStore(pricesDao, assetsDao)

    @Provides
    @Singleton
    fun provideGemPriceService(priceStore: GemstonePriceStore): GemPriceService = GemPriceService(priceStore)

    @Provides
    fun provideGemAssetDetailsServiceInterface(service: GemAssetDetailsService): GemAssetDetailsServiceInterface = service

    @Provides
    fun provideGemAssetsServiceInterface(service: GemAssetsService): GemAssetsServiceInterface = service
}
