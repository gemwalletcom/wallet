package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.data.coordinators.transaction.GetTransactionsImpl
import com.gemwallet.android.data.services.store.queries.TransactionsQuery
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
    fun provideGetTransactions(getSession: GetSession, getCurrentWalletId: GetCurrentWalletId, transactionsQuery: TransactionsQuery): GetTransactions = GetTransactionsImpl(getSession, getCurrentWalletId, transactionsQuery)
}
