package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.AssetsSearchService
import com.gemwallet.android.data.repositories.assets.CurrencyRatesService
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.ExponentialReconnection
import com.gemwallet.android.data.repositories.stream.StreamEventHandler
import com.gemwallet.android.data.repositories.support.SupportTypingState
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.InAppNotificationsDao
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.SupportMessagesDao
import com.gemwallet.android.data.services.gemapi.http.DeviceRequestSigner
import com.gemwallet.android.data.services.gemapi.http.GemDeviceRequestSigner
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
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
        pricesDao: PricesDao,
        sessionRepository: SessionRepository,
        balancesService: BalancesService,
        searchTokensCase: SearchTokensCase,
        streamSubscriptionService: StreamSubscriptionService,
        availabilityService: AssetsAvailabilityService,
        currencyRatesService: CurrencyRatesService,
        searchService: AssetsSearchService,
        recentAssetsService: RecentAssetsService,
    ): AssetsRepository = AssetsRepository(
        assetsDao = assetsDao,
        balancesDao = balancesDao,
        pricesDao = pricesDao,
        sessionRepository = sessionRepository,
        balancesService = balancesService,
        searchTokensCase = searchTokensCase,
        streamSubscriptionService = streamSubscriptionService,
        availabilityService = availabilityService,
        currencyRatesService = currencyRatesService,
        searchService = searchService,
        recentAssetsService = recentAssetsService,
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
    ): UpdateBalances = UpdateBalances(
        balancesDao = balancesDao,
        balancesService = balancesService,
    )

    @Provides
    @Singleton
    fun provideStreamEventHandler(
        pricesDao: PricesDao,
        sessionRepository: SessionRepository,
        syncTransactions: dagger.Lazy<SyncTransactions>,
        syncNfts: SyncNfts,
        updatePriceAlerts: UpdatePriceAlerts,
        syncFiatTransactions: dagger.Lazy<SyncFiatTransactions>,
        walletsRepository: WalletsRepository,
        assetsDao: AssetsDao,
        updateBalances: UpdateBalances,
        inAppNotificationsDao: InAppNotificationsDao,
        supportMessagesDao: SupportMessagesDao,
        supportTypingState: SupportTypingState,
    ): StreamEventHandler = StreamEventHandler(
        pricesDao = pricesDao,
        sessionRepository = sessionRepository,
        syncTransactions = syncTransactions,
        syncNfts = syncNfts,
        updatePriceAlerts = updatePriceAlerts,
        syncFiatTransactions = syncFiatTransactions,
        walletsRepository = walletsRepository,
        assetsDao = assetsDao,
        updateBalances = updateBalances,
        inAppNotificationsDao = inAppNotificationsDao,
        supportMessagesDao = supportMessagesDao,
        supportTypingState = supportTypingState,
    )

    @Provides
    @Singleton
    fun provideStreamSubscriptionService(
        assetsDao: AssetsDao,
        priceAlertsDao: PriceAlertsDao,
    ): StreamSubscriptionService = StreamSubscriptionService(
        assetsDao = assetsDao,
        priceAlertsDao = priceAlertsDao,
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
        userConfig: com.gemwallet.android.data.repositories.config.UserConfig,
        syncAssets: SyncAssets,
        syncPerpetuals: com.gemwallet.android.application.perpetual.coordinators.SyncPerpetuals,
        syncPerpetualPositions: com.gemwallet.android.application.perpetual.coordinators.SyncPerpetualPositions,
        deviceRequestSigner: DeviceRequestSigner,
        streamSubscriptionService: StreamSubscriptionService,
        eventHandler: StreamEventHandler,
    ): StreamObserverService = StreamObserverService(
        sessionRepository = sessionRepository,
        userConfig = userConfig,
        syncAssets = syncAssets,
        syncPerpetuals = syncPerpetuals,
        syncPerpetualPositions = syncPerpetualPositions,
        deviceRequestSigner = deviceRequestSigner,
        subscriptionService = streamSubscriptionService,
        eventHandler = eventHandler,
        reconnection = ExponentialReconnection(maxDelay = 30.0),
    )

    @Provides
    @Singleton
    fun providePerpetualRemoteSource(
        gateway: GemGateway,
    ): PerpetualService = PerpetualService(
        gateway = gateway,
    )
}
