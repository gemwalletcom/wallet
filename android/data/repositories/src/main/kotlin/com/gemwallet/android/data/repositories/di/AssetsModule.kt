package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.Constants
import com.gemwallet.android.application.assets.cases.SyncAssets
import com.gemwallet.android.application.tokens.cases.SearchTokens
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.repositories.stream.ExponentialReconnection
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import com.gemwallet.android.data.repositories.stream.WebSocketConnection
import com.gemwallet.android.data.repositories.stream.WebSocketRequest
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.math.fromHex
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.GemDeviceRequestSigner
import com.gemwallet.android.data.repositories.gemstone.GemstoneAssetStore
import com.gemwallet.android.data.repositories.gemstone.GemstonePortfolioStore
import uniffi.gemstone.GemApiClient
import uniffi.gemstone.GemAssetStore
import uniffi.gemstone.GemAssetsService
import com.gemwallet.android.data.repositories.gemstone.GemstoneBalanceStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import dagger.Lazy
import uniffi.gemstone.GemBalanceService
import com.gemwallet.android.data.repositories.gemstone.GemstonePriceStore
import com.gemwallet.android.data.service.store.database.PricesDao
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemPortfolioStore
import uniffi.gemstone.GemPriceAlertStore
import uniffi.gemstone.GemPriceService
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
import com.gemwallet.android.data.repositories.gemstone.GemstoneStreamConnection
import com.gemwallet.android.data.repositories.stream.WebSocketConnectable
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
    ): WebSocketConnectable = WebSocketConnection(
        client = okHttpClient,
        requestProvider = {
            WebSocketRequest(
                url = Constants.DEVICE_STREAM_WEBSOCKET_URL,
                headers = mapOf("Authorization" to deviceRequestSigner.get().sign("GET", Constants.DEVICE_STREAM_PATH, "", ByteArray(0))),
            )
        },
        reconnection = ExponentialReconnection(maxDelay = 30.0),
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
    fun provideGemAssetsService(
        apiClient: GemApiClient,
        gateway: GemGateway,
        assetStore: GemAssetStore,
        priceService: GemPriceService,
        preferencesService: GemPreferencesService,
    ): GemAssetsService = GemAssetsService(apiClient, gateway, assetStore, priceService, preferencesService)

    @Provides
    @Singleton
    fun provideGemstonePriceStore(pricesDao: PricesDao, assetsDao: AssetsDao): GemstonePriceStore = GemstonePriceStore(pricesDao, assetsDao)

    @Provides
    @Singleton
    fun provideGemPriceService(apiClient: GemApiClient, priceStore: GemstonePriceStore): GemPriceService = GemPriceService(apiClient, priceStore)
}
