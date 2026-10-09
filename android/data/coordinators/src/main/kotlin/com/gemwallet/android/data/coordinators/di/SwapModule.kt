package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.data.services.gemstone.stores.GemstoneSwapStore
import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.TransactionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemConfigService
import uniffi.gemstone.GemNodeService
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemStreamSubscriptionService
import uniffi.gemstone.GemSwapQuoteService
import uniffi.gemstone.GemSwapQuoteServiceInterface
import uniffi.gemstone.GemSwapService
import uniffi.gemstone.GemSwapServiceInterface
import uniffi.gemstone.GemSwapper
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object SwapModule {

    @Singleton
    @Provides
    fun provideGemSwapper(alienProvider: AlienProvider, nodes: GemNodeService): GemSwapper = GemSwapper(alienProvider, nodes)

    @Singleton
    @Provides
    fun provideGemSwapService(swapper: GemSwapper, assetsDao: AssetsDao, transactionsDao: TransactionsDao): GemSwapService = GemSwapService(swapper, GemstoneSwapStore(assetsDao, transactionsDao))

    @Provides
    fun provideGemSwapServiceInterface(service: GemSwapService): GemSwapServiceInterface = service

    @Singleton
    @Provides
    fun provideGemSwapQuoteService(
        swapService: GemSwapService,
        assetsService: GemAssetsService,
        preferencesService: GemPreferencesService,
        balanceService: GemBalanceService,
        streamSubscriptionService: GemStreamSubscriptionService,
        walletSessionService: GemWalletSessionService,
        configService: GemConfigService,
    ): GemSwapQuoteServiceInterface = GemSwapQuoteService(
        swap = swapService,
        assets = assetsService,
        preferences = preferencesService,
        balances = balanceService,
        stream = streamSubscriptionService,
        session = walletSessionService,
        config = configService,
    )
}
