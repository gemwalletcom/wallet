package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.application.assets.cases.SyncAssets
import com.gemwallet.android.data.services.gemstone.assets.AssetsAvailabilityService
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.stream.StreamObserverService
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnection
import com.gemwallet.android.data.services.gemstone.stream.WebSocketRequest
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.math.fromHex
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.GemDeviceRequestSigner
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.data.services.gemstone.stores.GemstonePortfolioStore
import uniffi.gemstone.GemApiClient
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemAssetDetailsService
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.GemSwapService
import uniffi.gemstone.GemExplorerService
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemBannerService
import com.gemwallet.android.data.services.gemstone.stores.GemstoneBalanceStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import dagger.Lazy
import uniffi.gemstone.GemBalanceService
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceStore
import com.gemwallet.android.data.service.store.database.PricesDao
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPortfolioStore
import uniffi.gemstone.GemPriceAlertStore
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemAddAssetService
import uniffi.gemstone.GemAddAssetServiceInterface
import uniffi.gemstone.GemReceiveService
import uniffi.gemstone.GemReceiveServiceInterface
import uniffi.gemstone.GemSupportStore
import uniffi.gemstone.GemNotificationStore
import uniffi.gemstone.GemFiatService
import uniffi.gemstone.GemPerpetualService
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemPriceAlertService
import uniffi.gemstone.GemStreamService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.OkHttpClient
import uniffi.gemstone.GemGateway
import javax.inject.Singleton
import com.gemwallet.android.data.services.gemstone.stream.GemstoneStreamConnection
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnectable
import uniffi.gemstone.GemStreamSubscriptionService
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemDeviceKeyService

@InstallIn(SingletonComponent::class)
@Module
object AssetsModule {

    @Provides
    @Singleton
    fun provideGemBalanceService(
        gateway: GemGateway,
        walletStore: GemstoneWalletStore,
        assetStore: GemAssetStore,
        assetsDao: AssetsDao,
        balancesDao: BalancesDao,
        assetsService: GemAssetsService,
        priceService: GemPriceService,
        streamSubscriptionService: GemStreamSubscriptionService,
        preferencesService: GemPreferencesService,
        transactionRunner: StoreTransactionRunner,
    ): GemBalanceService = GemBalanceService(
        gateway,
        walletStore,
        assetStore,
        GemstoneBalanceStore(balancesDao, assetsDao, transactionRunner),
        assetsService,
        priceService,
        streamSubscriptionService,
        preferencesService,
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
        walletStore: GemstoneWalletStore,
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
        walletStore,
    )

    @Provides
    @Singleton
    fun provideStreamConnection(
        deviceRequestSigner: Lazy<GemDeviceRequestSigner>,
        okHttpClient: OkHttpClient,
        connectionService: GemConnectionService,
    ): WebSocketConnectable = WebSocketConnection(
        client = okHttpClient,
        requestProvider = {
            val stream = deviceRequestSigner.get().deviceStreamRequest()
            WebSocketRequest(url = stream.url, headers = mapOf("Authorization" to stream.authorization))
        },
        connectionService = connectionService,
    )

    @Provides
    @Singleton
    fun provideStreamSubscriptionService(
        priceService: GemPriceService,
        priceAlertStore: GemPriceAlertStore,
        connection: WebSocketConnectable,
    ): GemStreamSubscriptionService = GemStreamSubscriptionService(
        price = priceService,
        alerts = priceAlertStore,
        connection = GemstoneStreamConnection(connection),
    )

    @Provides
    @Singleton
    fun provideDeviceRequestSigner(
        deviceKeyService: GemDeviceKeyService,
    ): GemDeviceRequestSigner = runBlocking {
        GemDeviceRequestSigner(deviceKeyService.keyPair().privateKey)
    }

    @Provides
    @Singleton
    fun provideStreamObserverService(
        getSession: GetSession,
        getCurrentCurrency: GetCurrentCurrency,
        syncAssets: SyncAssets,
        streamSubscriptionService: GemStreamSubscriptionService,
        streamService: GemStreamService,
        connection: WebSocketConnectable,
        deviceService: GemDeviceService,
    ): StreamObserverService = StreamObserverService(
        getSession = getSession,
        getCurrentCurrency = getCurrentCurrency,
        syncAssets = syncAssets,
        subscriptionService = streamSubscriptionService,
        streamService = streamService,
        connection = connection,
        deviceService = deviceService,
    )

    @Provides
    @Singleton
    fun provideGemstoneAssetStore(
        assetsDao: AssetsDao,
        availabilityService: AssetsAvailabilityService,
    ): GemstoneAssetStore = GemstoneAssetStore(assetsDao, availabilityService)

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
    )

    @Provides
    @Singleton
    fun provideGemAssetsService(
        apiClient: GemApiClient,
        gateway: GemGateway,
        assetStore: GemAssetStore,
        priceService: GemPriceService,
        preferencesService: GemPreferencesService,
    ): GemAssetsService = GemAssetsService(apiClient, gateway, assetStore, priceService, preferencesService)

    @Provides
    fun provideGemReceiveService(
        balanceService: GemBalanceService,
        assetsService: GemAssetsService,
    ): GemReceiveServiceInterface = GemReceiveService(balanceService, assetsService)

    @Provides
    fun provideGemAddAssetService(
        assetsService: GemAssetsService,
        balanceService: GemBalanceService,
        explorerService: GemExplorerService,
    ): GemAddAssetServiceInterface = GemAddAssetService(assetsService, balanceService, explorerService)

    @Provides
    @Singleton
    fun provideGemstonePriceStore(pricesDao: PricesDao, assetsDao: AssetsDao): GemstonePriceStore = GemstonePriceStore(pricesDao, assetsDao)

    @Provides
    @Singleton
    fun provideGemPriceService(apiClient: GemApiClient, priceStore: GemstonePriceStore): GemPriceService = GemPriceService(apiClient, priceStore)
}
