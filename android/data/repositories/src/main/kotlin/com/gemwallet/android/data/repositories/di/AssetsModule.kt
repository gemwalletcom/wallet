package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.device.coordinators.GetDeviceId
import com.gemwallet.android.application.fiat.coordinators.SyncFiatTransactions
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.blockchain.services.AddressStatusService
import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.blockchain.services.PerpetualService
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.UpdateBalances
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.StreamEventHandler
import com.gemwallet.android.data.repositories.stream.StreamObserverService
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PriceAlertsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.services.gemapi.http.DeviceRequestSigner
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
        assetsPriorityDao: AssetsPriorityDao,
        balancesDao: BalancesDao,
        pricesDao: PricesDao,
        sessionRepository: SessionRepository,
        balancesService: BalancesService,
        getChangedTransactions: GetChangedTransactions,
        searchTokensCase: SearchTokensCase,
        streamSubscriptionService: StreamSubscriptionService,
    ): AssetsRepository = AssetsRepository(
        assetsDao = assetsDao,
        assetsPriorityDao = assetsPriorityDao,
        balancesDao = balancesDao,
        pricesDao = pricesDao,
        sessionRepository = sessionRepository,
        getChangedTransactions = getChangedTransactions,
        balancesService = balancesService,
        searchTokensCase = searchTokensCase,
        streamSubscriptionService = streamSubscriptionService,
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
    fun provideAddressStatusService(
        gateway: GemGateway,
    ): AddressStatusService = AddressStatusService(
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
    ): DeviceRequestSigner = DeviceRequestSigner(
        getDeviceId = getDeviceId,
    )

    @Provides
    @Singleton
    fun provideStreamObserverService(
        sessionRepository: SessionRepository,
        deviceRequestSigner: DeviceRequestSigner,
        streamSubscriptionService: StreamSubscriptionService,
        eventHandler: StreamEventHandler,
    ): StreamObserverService = StreamObserverService(
        sessionRepository = sessionRepository,
        deviceRequestSigner = deviceRequestSigner,
        subscriptionService = streamSubscriptionService,
        eventHandler = eventHandler,
    )

    @Provides
    @Singleton
    fun providePerpetualRemoteSource(
        gateway: GemGateway,
    ): PerpetualService = PerpetualService(
        gateway = gateway,
    )
}
