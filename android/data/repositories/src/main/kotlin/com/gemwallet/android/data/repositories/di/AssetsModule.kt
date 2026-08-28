package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.Constants
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.prices.PricesRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.ExponentialReconnection
import com.gemwallet.android.data.repositories.support.SupportChatRepository
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

@InstallIn(SingletonComponent::class)
@Module
object AssetsModule {
    @Provides
    @Singleton
    fun provideAssetsRepository(
        assetsDao: AssetsDao,
        pricesRepository: PricesRepository,
        sessionRepository: SessionRepository,
        searchTokensCase: SearchTokensCase,
        streamSubscriptionService: GemStreamSubscriptionService,
        updateBalances: UpdateBalances,
    ): AssetsRepository = AssetsRepository(
        assetsDao = assetsDao,
        pricesRepository = pricesRepository,
        sessionRepository = sessionRepository,
        searchTokensCase = searchTokensCase,
        streamSubscriptionService = streamSubscriptionService,
        updateBalances = updateBalances,
    )


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
    fun provideUpdateBalances(
        balanceService: GemBalanceService,
    ): UpdateBalances = UpdateBalances(balanceService)

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
        getDeviceId: GetDeviceId,
    ): GemDeviceRequestSigner = runBlocking {
        GemDeviceRequestSigner(getDeviceId.getDeviceKey().fromHex())
    }

    @Provides
    @Singleton
    fun provideStreamObserverService(
        sessionRepository: SessionRepository,
        syncAssets: SyncAssets,
        streamSubscriptionService: GemStreamSubscriptionService,
        streamService: GemStreamService,
        connection: WebSocketConnectable,
        syncDevice: SyncDevice,
    ): StreamObserverService = StreamObserverService(
        sessionRepository = sessionRepository,
        syncAssets = syncAssets,
        subscriptionService = streamSubscriptionService,
        streamService = streamService,
        connection = connection,
        syncDevice = syncDevice,
    )

    @Provides
    @Singleton
    fun provideGemAssetStore(
        assetsDao: AssetsDao,
        availabilityService: AssetsAvailabilityService,
    ): GemAssetStore = GemstoneAssetStore(assetsDao, availabilityService)

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
    fun provideGemPriceService(
        apiClient: GemApiClient,
        pricesDao: PricesDao,
        assetsDao: AssetsDao,
    ): GemPriceService = GemPriceService(apiClient, GemstonePriceStore(pricesDao, assetsDao))
}
