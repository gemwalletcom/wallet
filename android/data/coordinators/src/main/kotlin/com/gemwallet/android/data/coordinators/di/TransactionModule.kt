package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.SyncAssetTransactions
import com.gemwallet.android.application.transactions.cases.SyncTransactions
import com.gemwallet.android.data.coordinators.transaction.GetTransactionDetailsImpl
import com.gemwallet.android.data.coordinators.transaction.GetTransactionsImpl
import com.gemwallet.android.data.coordinators.transaction.SyncTransactionsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.application.transactions.cases.GetPendingTransactionsCount
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.data.coordinators.transaction.ClearPendingTransactionsImpl
import com.gemwallet.android.data.coordinators.transaction.GetPendingTransactionsCountImpl
import com.gemwallet.android.data.coordinators.transaction.GetTransactionImpl
import com.gemwallet.android.data.service.store.database.TransactionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TransactionModule {
    @Provides
    @Singleton
    fun provideGetTransactions(
        sessionRepository: SessionRepository,
        transactionsDao: TransactionsDao,
    ): GetTransactions {
        return GetTransactionsImpl(sessionRepository, transactionsDao)
    }

    @Provides
    @Singleton
    fun provideGetTransaction(
        sessionRepository: SessionRepository,
        transactionsDao: TransactionsDao,
    ): GetTransaction = GetTransactionImpl(sessionRepository, transactionsDao)

    @Provides
    @Singleton
    fun provideGetPendingTransactionsCount(
        sessionRepository: SessionRepository,
        transactionsDao: TransactionsDao,
    ): GetPendingTransactionsCount = GetPendingTransactionsCountImpl(sessionRepository, transactionsDao)

    @Provides
    @Singleton
    fun provideClearPending(transactionsDao: TransactionsDao): ClearPendingTransactions = ClearPendingTransactionsImpl(transactionsDao)

    @Provides
    @Singleton
    fun provideSyncTransactions(
        syncTransactionsImpl: SyncTransactionsImpl,
    ): SyncTransactions = syncTransactionsImpl

    @Provides
    @Singleton
    fun provideSyncAssetTransactions(
        syncTransactionsImpl: SyncTransactionsImpl,
    ): SyncAssetTransactions = syncTransactionsImpl

    @Provides
    @Singleton
    fun provideGetTransactionDetails(
        sessionRepository: SessionRepository,
        getTransaction: GetTransaction,
        assetsRepository: AssetsRepository,
        explorerService: GemExplorerService,
    ): GetTransactionDetails {
        return GetTransactionDetailsImpl(
            sessionRepository = sessionRepository,
            getTransaction = getTransaction,
            assetsRepository = assetsRepository,
            explorerService = explorerService,
        )
    }
}
