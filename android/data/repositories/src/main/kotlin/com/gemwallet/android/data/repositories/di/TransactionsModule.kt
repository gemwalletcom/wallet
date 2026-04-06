package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.GetTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.transactions.TransactionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionsRepositoryImpl
import com.gemwallet.android.data.service.store.database.TransactionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemGateway
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TransactionsModule {

    @Singleton
    @Provides
    fun provideTransactionsRepository(
        transactionsDao: TransactionsDao,
        gateway: GemGateway,
    ): TransactionsRepositoryImpl = TransactionsRepositoryImpl(
        transactionsDao = transactionsDao,
        transactionStatusService = TransactionStatusService(
            gateway = gateway,
        ),
    )

    @Singleton
    @Provides
    fun provideTransactionRepository( // TODO: Remove when TransactionsRepositoryImpl will refactored
        impl: TransactionsRepositoryImpl
    ): TransactionRepository = impl

    @Singleton
    @Provides
    fun provideGetChangedTransactions(transactionsRepository: TransactionsRepositoryImpl): GetChangedTransactions {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideGetPendingTransactionsCount(transactionsRepository: TransactionsRepositoryImpl): GetPendingTransactionsCount {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideGetTransactionCase(transactionsRepository: TransactionsRepositoryImpl): GetTransaction {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideSaveTransactionsCase(transactionsRepository: TransactionsRepositoryImpl): SaveTransactions {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideCreateTransactionsCase(transactionsRepository: TransactionsRepositoryImpl): CreateTransaction {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideClearPending(transactionsRepository: TransactionsRepositoryImpl): ClearPendingTransactions {
        return transactionsRepository
    }
}

