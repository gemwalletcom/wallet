package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.swap.cases.RequestSwapQuotes
import com.gemwallet.android.data.coordinators.swap.RequestSwapQuotesImpl
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneKeystorePassword
import com.gemwallet.android.data.services.gemstone.stores.GemstoneSwapStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemKeystore
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
    fun provideGemSwapService(gemSwapper: GemSwapper, gemKeystore: GemKeystore, passwordStore: PasswordStore, assetsDao: AssetsDao, transactionsDao: TransactionsDao): GemSwapService = GemSwapService(
        swapper = gemSwapper,
        keystore = gemKeystore,
        password = GemstoneKeystorePassword(passwordStore),
        store = GemstoneSwapStore(assetsDao, transactionsDao),
    )

    @Provides
    fun provideGemSwapServiceInterface(service: GemSwapService): GemSwapServiceInterface = service

    @Singleton
    @Provides
    fun provideGemSwapQuoteService(
        swapService: GemSwapServiceInterface,
        preferencesService: GemPreferencesService,
        balanceService: GemBalanceService,
        streamSubscriptionService: GemStreamSubscriptionService,
        walletSessionService: GemWalletSessionService,
    ): GemSwapQuoteServiceInterface = GemSwapQuoteService(
        swap = swapService as GemSwapService,
        preferences = preferencesService,
        balances = balanceService,
        stream = streamSubscriptionService,
        session = walletSessionService,
    )

    @Singleton
    @Provides
    fun provideRequestSwapQuotes(swapService: GemSwapQuoteServiceInterface): RequestSwapQuotes = RequestSwapQuotesImpl(swapService)
}
