package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.Constants
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.notifications.InAppNotificationsRepository
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
        balancesService: BalancesService,
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
        balancesService = balancesService,
        searchTokensCase = searchTokensCase,
        streamSubscriptionService = streamSubscriptionService,
        availabilityService = availabilityService,
        currencyRatesService = currencyRatesService,
        updateBalances = updateBalances,
    )

    @Provides
    @Singleton
    fun provideBalanceRemoteSource(
        gateway: GemGateway,
    ): BalancesService = BalancesService(
        gateway = gateway,
    )

    @Provides
    @Singleton
    fun provideUpdateBalances(
        balancesDao: BalancesDao,
        balancesService: BalancesService,
        assetsDao: AssetsDao,
    ): UpdateBalances = UpdateBalances(
        balancesDao = balancesDao,
        balancesService = balancesService,
        assetsDao = assetsDao,
    )

    @Provides
    @Singleton
    fun provideStreamEventHandler(
        pricesRepository: PricesRepository,
        syncTransactions: dagger.Lazy<SyncTransactions>,
        syncNfts: SyncNfts,
        updatePriceAlerts: UpdatePriceAlerts,
        syncFiatTransactions: dagger.Lazy<SyncFiatTransactions>,
        walletsRepository: WalletsRepository,
        updateBalances: UpdateBalances,
        inAppNotificationsRepository: InAppNotificationsRepository,
        supportChatRepository: SupportChatRepository,
    ): StreamEventHandler = StreamEventHandler(
        pricesRepository = pricesRepository,
        syncTransactions = syncTransactions,
        syncNfts = syncNfts,
        updatePriceAlerts = updatePriceAlerts,
        syncFiatTransactions = syncFiatTransactions,
        walletsRepository = walletsRepository,
        updateBalances = updateBalances,
        inAppNotificationsRepository = inAppNotificationsRepository,
        supportChatRepository = supportChatRepository,
    )

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
}
