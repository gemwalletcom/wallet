package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.SyncAssetTransactions
import com.gemwallet.android.application.transactions.cases.SyncTransactions
import com.gemwallet.android.data.coordinators.transaction.GetTransactionDetailsImpl
import com.gemwallet.android.data.coordinators.transaction.GetTransactionsImpl
import com.gemwallet.android.data.coordinators.transaction.SyncTransactionsImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.application.transactions.cases.GetPendingTransactionsCount
import com.gemwallet.android.cases.transactions.ClearPendingTransactions
import com.gemwallet.android.data.coordinators.transaction.ClearPendingTransactionsImpl
import com.gemwallet.android.data.coordinators.transaction.GetPendingTransactionsCountImpl
import com.gemwallet.android.data.coordinators.transaction.GetTransactionImpl
import com.gemwallet.android.data.repositories.gemstone.GemstoneTransactionStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.application.assets.cases.GetWalletAssets

@InstallIn(SingletonComponent::class)
@Module
object TransactionModule {
    @Provides
    @Singleton
    fun provideGetTransactions(
        sessionRepository: SessionRepository,
        transactionStore: GemstoneTransactionStore,
    ): GetTransactions {
        return GetTransactionsImpl(sessionRepository, transactionStore)
    }

    @Provides
    @Singleton
    fun provideGetTransaction(
        sessionRepository: SessionRepository,
        transactionStore: GemstoneTransactionStore,
    ): GetTransaction = GetTransactionImpl(sessionRepository, transactionStore)

    @Provides
    @Singleton
    fun provideGetPendingTransactionsCount(
        sessionRepository: SessionRepository,
        transactionStore: GemstoneTransactionStore,
    ): GetPendingTransactionsCount = GetPendingTransactionsCountImpl(sessionRepository, transactionStore)

    @Provides
    @Singleton
    fun provideClearPending(transactionStore: GemstoneTransactionStore): ClearPendingTransactions = ClearPendingTransactionsImpl(transactionStore)

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
        getWalletAssets: GetWalletAssets,
        explorerService: GemExplorerService,
    ): GetTransactionDetails {
        return GetTransactionDetailsImpl(
            sessionRepository = sessionRepository,
            getTransaction = getTransaction,
            getWalletAssets = getWalletAssets,
            explorerService = explorerService,
        )
    }
}
