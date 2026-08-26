package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.Constants
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.pricealerts.PriceAlertRepository
import com.gemwallet.android.data.repositories.prices.PricesRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.ExponentialReconnection
import com.gemwallet.android.data.repositories.stream.StreamEventHandler
import com.gemwallet.android.data.repositories.support.SupportChatRepository
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.repositories.stream.WebSocketConnection
import com.gemwallet.android.data.repositories.stream.WebSocketRequest
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.services.gemapi.http.DeviceRequestSigner
import com.gemwallet.android.data.services.gemapi.http.GemDeviceRequestSigner
import com.gemwallet.android.data.repositories.assets.GemstoneAssetStore
import uniffi.gemstone.GemApiClient
import uniffi.gemstone.GemAssetsService
import com.gemwallet.android.data.repositories.assets.GemstoneBalanceStore
import com.gemwallet.android.data.repositories.wallets.GemstoneWalletStore
import dagger.Lazy
import uniffi.gemstone.GemBalanceService
import com.gemwallet.android.data.repositories.prices.GemstonePriceStore
import com.gemwallet.android.data.service.store.database.PricesDao
import uniffi.gemstone.GemPreferencesService
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

@InstallIn(SingletonComponent::class)
@Module
object AssetsModule {
    @Provides
    @Singleton
    fun provideAssetsRepository(
        assetsDao: AssetsDao,
        balancesDao: BalancesDao,
        pricesRepository: PricesRepository,
        sessionRepository: SessionRepository,
        searchTokensCase: SearchTokensCase,
        streamSubscriptionService: StreamSubscriptionService,
        availabilityService: AssetsAvailabilityService,
        currencyRatesService: CurrencyRatesService,
        updateBalances: UpdateBalances,
    ): AssetsRepository = AssetsRepository(
        assetsDao = assetsDao,
        balancesDao = balancesDao,
        pricesRepository = pricesRepository,
        sessionRepository = sessionRepository,
        searchTokensCase = searchTokensCase,
        streamSubscriptionService = streamSubscriptionService,
        availabilityService = availabilityService,
        currencyRatesService = currencyRatesService,
        updateBalances = updateBalances,
    )


    @Provides
    @Singleton
    fun provideGemBalanceService(
        gateway: GemGateway,
        walletsRepository: Lazy<WalletsRepository>,
        assetsDao: AssetsDao,
        balancesDao: BalancesDao,
        availabilityService: AssetsAvailabilityService,
    ): GemBalanceService = GemBalanceService(
        gateway,
        GemstoneWalletStore(walletsRepository),
        GemstoneAssetStore(assetsDao, availabilityService),
        GemstoneBalanceStore(balancesDao),
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
        walletsRepository: Lazy<WalletsRepository>,
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
        GemstoneWalletStore(walletsRepository),
    )

    @Provides
    @Singleton
    fun provideStreamEventHandler(
        streamService: GemStreamService,
        sessionRepository: SessionRepository,
        supportChatRepository: SupportChatRepository,
    ): StreamEventHandler = StreamEventHandler(streamService, sessionRepository, supportChatRepository)

    @Provides
    @Singleton
    fun provideStreamSubscriptionService(
        assetsDao: AssetsDao,
        priceAlertRepository: PriceAlertRepository,
    ): StreamSubscriptionService = StreamSubscriptionService(
        assetsDao = assetsDao,
        priceAlertRepository = priceAlertRepository,
    )

    @Provides
    @Singleton
    fun provideDeviceRequestSigner(
        getDeviceId: GetDeviceId,
    ): DeviceRequestSigner = GemDeviceRequestSigner(
        getDeviceId = getDeviceId,
    )

    @Provides
    @Singleton
    fun provideStreamObserverService(
        sessionRepository: SessionRepository,
        syncAssets: SyncAssets,
        deviceRequestSigner: DeviceRequestSigner,
        streamSubscriptionService: StreamSubscriptionService,
        eventHandler: StreamEventHandler,
        syncDevice: SyncDevice,
        okHttpClient: OkHttpClient,
    ): StreamObserverService = StreamObserverService(
        sessionRepository = sessionRepository,
        syncAssets = syncAssets,
        subscriptionService = streamSubscriptionService,
        eventHandler = eventHandler,
        connection = WebSocketConnection(
            client = okHttpClient,
            requestProvider = {
                WebSocketRequest(
                    url = Constants.DEVICE_STREAM_WEBSOCKET_URL,
                    headers = deviceRequestSigner.sign("GET", Constants.DEVICE_STREAM_PATH).toHeaders(),
                )
            },
            reconnection = ExponentialReconnection(maxDelay = 30.0),
        ),
        syncDevice = syncDevice,
    )

    @Provides
    @Singleton
    fun providePerpetualRemoteSource(
        gateway: GemGateway,
    ): PerpetualService = PerpetualService(
        gateway = gateway,
    )

    @Provides
    @Singleton
    fun provideGemAssetsService(
        apiClient: GemApiClient,
        assetsDao: AssetsDao,
        availabilityService: AssetsAvailabilityService,
        priceService: GemPriceService,
        preferencesService: GemPreferencesService,
    ): GemAssetsService = GemAssetsService(apiClient, GemstoneAssetStore(assetsDao, availabilityService), priceService, preferencesService)

    @Provides
    @Singleton
    fun provideGemPriceService(
        apiClient: GemApiClient,
        pricesDao: PricesDao,
        assetsDao: AssetsDao,
    ): GemPriceService = GemPriceService(apiClient, GemstonePriceStore(pricesDao, assetsDao))
}
