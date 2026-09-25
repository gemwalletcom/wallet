package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.data.services.store.database.AddressesDao
import com.gemwallet.android.data.services.store.database.StoreTransactionRunner
import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAddressStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStateStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import com.gemwallet.android.data.services.gemstone.transactions.TransactionStatusService
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemBalanceService
import uniffi.gemstone.GemDeviceApiClient
import uniffi.gemstone.GemGateway
import uniffi.gemstone.GemNameService
import uniffi.gemstone.GemNftService
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemStakeService
import uniffi.gemstone.GemTransactionStateService
import uniffi.gemstone.GemTransactionStateServiceInterface
import uniffi.gemstone.GemTransactionsService
import uniffi.gemstone.GemTransactionsServiceInterface
import uniffi.gemstone.GemWalletPreferencesService
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TransactionsModule {

    @Singleton
    @Provides
    fun provideTransactionsService(
        apiClient: GemDeviceApiClient,
        assetsService: GemAssetsService,
        transactionStore: GemstoneTransactionStateStore,
        nameService: GemNameService,
        walletPreferencesService: GemWalletPreferencesService,
        walletSessionService: GemWalletSessionService,
        tracker: TransactionStatusService,
    ): GemTransactionsService = GemTransactionsService(
        apiClient,
        assetsService,
        transactionStore,
        nameService,
        walletPreferencesService,
        walletSessionService,
        tracker,
    )

    @Singleton
    @Provides
    fun provideGemstoneTransactionStore(transactionsDao: TransactionsDao): GemstoneTransactionStore = GemstoneTransactionStore(transactionsDao)

    @Singleton
    @Provides
    fun provideTransactionStateStore(transactionsDao: TransactionsDao, transactionRunner: StoreTransactionRunner): GemstoneTransactionStateStore = GemstoneTransactionStateStore(transactionsDao, transactionRunner)

    @Singleton
    @Provides
    fun provideTransactionStateService(
        store: GemstoneTransactionStateStore,
        gateway: GemGateway,
        assetsService: GemAssetsService,
        balanceService: GemBalanceService,
        stakeService: GemStakeService,
        nftService: GemNftService,
        paymentService: GemPaymentService,
    ): GemTransactionStateService = GemTransactionStateService(gateway, store, assetsService, balanceService, stakeService, nftService, paymentService)

    @Singleton
    @Provides
    fun provideTransactionStatusService(stateService: GemTransactionStateService): TransactionStatusService = TransactionStatusService(stateService = stateService).also { stateService.setStatus(it) }

    @Provides
    fun provideGemTransactionsServiceInterface(service: GemTransactionsService): GemTransactionsServiceInterface = service

    @Provides
    @Singleton
    fun provideTransactionStateServiceInterface(service: GemTransactionStateService): GemTransactionStateServiceInterface = service
}
