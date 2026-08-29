package com.gemwallet.android.data.adapters.di

import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.data.adapters.transactions.TransactionStateTracker
import com.gemwallet.android.data.adapters.gemstone.GemstoneAddressStore
import com.gemwallet.android.data.adapters.gemstone.GemstoneTransactionStateStore
import com.gemwallet.android.data.adapters.gemstone.GemstoneTransactionStore
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
import javax.inject.Singleton
import uniffi.gemstone.GemWalletPreferencesService
import com.gemwallet.android.data.adapters.gemstone.GemstoneWalletStore

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
    ): GemTransactionsService = GemTransactionsService(
        apiClient,
        assetsService,
        transactionStore,
        addressStore,
        walletPreferencesService,
    )

    @Singleton
    @Provides
    fun provideGemstoneTransactionStore(
        transactionsDao: TransactionsDao,
        transactionRunner: StoreTransactionRunner,
    ): GemstoneTransactionStore = GemstoneTransactionStore(transactionsDao, transactionRunner)

    @Singleton
    @Provides
    fun provideTransactionStateService(
        transactionsDao: TransactionsDao,
        walletStore: GemstoneWalletStore,
        gateway: GemGateway,
        assetsService: GemAssetsService,
        balanceService: GemBalanceService,
        stakeService: GemStakeService,
        nftService: GemNftService,
        transactionRunner: StoreTransactionRunner,
    ): GemTransactionStateService = GemTransactionStateService(
        gateway,
        GemstoneTransactionStateStore(transactionsDao, walletStore, transactionRunner),
        assetsService,
        balanceService,
        stakeService,
        nftService,
    )

    @Singleton
    @Provides
    fun provideTransactionStateTracker(
        stateService: GemTransactionStateService,
    ): TransactionStateTracker = TransactionStateTracker(stateService = stateService)

    @Singleton
    @Provides
    fun provideCreateTransactionsCase(tracker: TransactionStateTracker): CreateTransaction = tracker

}
