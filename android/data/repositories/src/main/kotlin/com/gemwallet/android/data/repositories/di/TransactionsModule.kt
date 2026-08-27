package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionStateScheduler
import com.gemwallet.android.data.repositories.gemstone.GemstoneAddressStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStateStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStore
import com.gemwallet.android.data.repositories.transactions.TransactionsRepositoryImpl
import com.gemwallet.android.data.service.store.database.AddressesDao
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
        transactionsRepository: TransactionsRepositoryImpl,
        addressesDao: AddressesDao,
        walletPreferencesService: GemWalletPreferencesService,
    ): GemTransactionsService = GemTransactionsService(
        apiClient,
        assetsService,
        GemstoneTransactionStore(transactionsRepository),
        GemstoneAddressStore(addressesDao),
        walletPreferencesService,
    )

    @Singleton
    @Provides
    fun provideTransactionStateService(
        transactionsDao: TransactionsDao,
        walletsRepository: Lazy<WalletsRepository>,
        gateway: GemGateway,
        balanceService: GemBalanceService,
        stakeService: GemStakeService,
        nftService: GemNftService,
    ): GemTransactionStateService = GemTransactionStateService(
        gateway,
        GemstoneTransactionStateStore(transactionsDao, walletsRepository),
        balanceService,
        stakeService,
        nftService,
    )

    @Singleton
    @Provides
    fun provideTransactionStateScheduler(
        stateService: GemTransactionStateService,
    ): TransactionStateScheduler = TransactionStateScheduler(stateService = stateService)

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
    fun provideSaveTransactionsCase(transactionsRepository: TransactionsRepositoryImpl): SaveTransactions {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideCreateTransactionsCase(scheduler: TransactionStateScheduler): CreateTransaction = scheduler

    @Singleton
    @Provides
    fun provideClearPending(transactionsRepository: TransactionsRepositoryImpl): ClearPendingTransactions {
        return transactionsRepository
    }
}
