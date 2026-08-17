package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.transactions.coordinators.GetPendingTransactionsCount
import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.repositories.assets.TransactionPostProcessingService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionStateScheduler
import com.gemwallet.android.data.repositories.transactions.TransactionStateService
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
        sessionRepository: SessionRepository,
        transactionsDao: TransactionsDao,
    ): TransactionsRepositoryImpl = TransactionsRepositoryImpl(
        sessionRepository = sessionRepository,
        transactionsDao = transactionsDao,
    )

    @Singleton
    @Provides
    fun provideTransactionStateService(
        transactionsDao: TransactionsDao,
        gateway: GemGateway,
    ): TransactionStateService = TransactionStateService(
        transactionsDao = transactionsDao,
        transactionStatusService = TransactionStatusService(
            gateway = gateway,
        ),
    )

    @Singleton
    @Provides
    fun provideTransactionStateScheduler(
        sessionRepository: SessionRepository,
        transactionsDao: TransactionsDao,
        stateService: TransactionStateService,
        postProcessingService: TransactionPostProcessingService,
    ): TransactionStateScheduler = TransactionStateScheduler(
        sessionRepository = sessionRepository,
        transactionsDao = transactionsDao,
        stateService = stateService,
        postProcessingService = postProcessingService,
    )

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
    fun provideCreateTransactionsCase(transactionsRepository: TransactionsRepositoryImpl): CreateTransaction {
        return transactionsRepository
    }

    @Singleton
    @Provides
    fun provideClearPending(transactionsRepository: TransactionsRepositoryImpl): ClearPendingTransactions {
        return transactionsRepository
    }
}
