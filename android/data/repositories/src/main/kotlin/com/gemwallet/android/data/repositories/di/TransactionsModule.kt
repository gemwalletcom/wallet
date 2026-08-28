package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionStateTracker
import com.gemwallet.android.data.repositories.gemstone.GemstoneAddressStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStateStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStore
import com.gemwallet.android.data.repositories.transactions.TransactionsRepositoryImpl
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
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

@InstallIn(SingletonComponent::class)
@Module
object TransactionsModule {

    @Singleton
    @Provides
    fun provideTransactionsRepository(
        sessionRepository: SessionRepository,
        transactionsDao: TransactionsDao,
    ): TransactionsRepositoryImpl = TransactionsRepositoryImpl(
        sessionRepository = sessionRepository,
        transactionsDao = transactionsDao,
    )

    @Singleton
    @Provides
    fun provideTransactionsService(
        apiClient: GemDeviceApiClient,
        assetsService: GemAssetsService,
        transactionsDao: TransactionsDao,
        addressesDao: AddressesDao,
        walletPreferencesService: GemWalletPreferencesService,
        transactionRunner: StoreTransactionRunner,
    ): GemTransactionsService = GemTransactionsService(
        apiClient,
        assetsService,
        GemstoneTransactionStore(transactionsDao, transactionRunner),
        GemstoneAddressStore(addressesDao),
        walletPreferencesService,
    )

    @Singleton
    @Provides
    fun provideTransactionStateService(
        transactionsDao: TransactionsDao,
        walletsRepository: Lazy<WalletsRepository>,
        gateway: GemGateway,
        assetsService: GemAssetsService,
        balanceService: GemBalanceService,
        stakeService: GemStakeService,
        nftService: GemNftService,
        transactionRunner: StoreTransactionRunner,
    ): GemTransactionStateService = GemTransactionStateService(
        gateway,
        GemstoneTransactionStateStore(transactionsDao, walletsRepository, transactionRunner),
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
    fun provideTransactionRepository(
        impl: TransactionsRepositoryImpl
    ): TransactionRepository = impl

    @Singleton
    @Provides
    fun provideGetPendingTransactionsCount(transactionsRepository: TransactionsRepositoryImpl): GetPendingTransactionsCount {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideCreateTransactionsCase(tracker: TransactionStateTracker): CreateTransaction = tracker

    @Singleton
    @Provides
    fun provideClearPending(transactionsRepository: TransactionsRepositoryImpl): ClearPendingTransactions {
        return transactionsRepository
    }
}
