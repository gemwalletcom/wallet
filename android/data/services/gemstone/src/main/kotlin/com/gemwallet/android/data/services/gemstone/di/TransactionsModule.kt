package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.data.services.gemstone.transactions.TransactionStatusService
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStateStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.TransactionsDao
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemStakeService
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemTransactionsServiceInterface
import javax.inject.Singleton
import uniffi.gemstone.GemPreferencesService
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletSessionService
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore

@InstallIn(SingletonComponent::class)
@Module
object TransactionsModule {

    @Singleton
    @Provides
    fun provideTransactionsService(
        apiClient: GemDeviceApiClient,
        assetsService: GemAssetsService,
        transactionStore: GemstoneTransactionStore,
        addressStore: GemstoneAddressStore,
        walletPreferencesService: GemWalletPreferencesService,
        preferencesService: GemPreferencesService,
        walletSessionService: GemWalletSessionService,
        tracker: TransactionStatusService,
    ): GemTransactionsService = GemTransactionsService(
        apiClient,
        assetsService,
        transactionStore,
        addressStore,
        walletPreferencesService,
        preferencesService,
        walletSessionService,
        tracker,
    )

    @Singleton
    @Provides
    fun provideGemstoneTransactionStore(
        transactionsDao: TransactionsDao,
        transactionRunner: StoreTransactionRunner,
    ): GemstoneTransactionStore = GemstoneTransactionStore(transactionsDao, transactionRunner)

    @Singleton
    @Provides
    fun provideTransactionStateStore(
        transactionsDao: TransactionsDao,
        walletStore: GemstoneWalletStore,
        transactionRunner: StoreTransactionRunner,
    ): GemstoneTransactionStateStore = GemstoneTransactionStateStore(transactionsDao, walletStore, transactionRunner)

    @Singleton
    @Provides
    fun provideTransactionStateService(
        store: GemstoneTransactionStateStore,
        gateway: GemGateway,
        assetsService: GemAssetsService,
        balanceService: GemBalanceService,
        stakeService: GemStakeService,
        nftService: GemNftService,
    ): GemTransactionStateService = GemTransactionStateService(gateway, store, assetsService, balanceService, stakeService, nftService)

    @Singleton
    @Provides
    fun provideTransactionStatusService(
        stateService: GemTransactionStateService,
    ): TransactionStatusService = TransactionStatusService(stateService = stateService)

    @Singleton
    @Provides
    fun provideCreateTransactionsCase(tracker: TransactionStatusService): CreateTransaction = tracker

    @Provides
    fun provideGemTransactionsServiceInterface(service: GemTransactionsService): GemTransactionsServiceInterface = service
}
