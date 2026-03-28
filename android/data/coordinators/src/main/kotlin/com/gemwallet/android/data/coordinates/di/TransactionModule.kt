package com.gemwallet.android.data.coordinates.di

import com.gemwallet.android.application.transactions.coordinators.GetTransactionDetails
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.coordinates.transaction.GetTransactionDetailsImpl
import com.gemwallet.android.data.coordinates.transaction.GetTransactionsImpl
import com.gemwallet.android.data.repositoreis.assets.AssetsRepository
import com.gemwallet.android.data.repositoreis.session.SessionRepository
import com.gemwallet.android.data.repositoreis.transactions.TransactionRepository
import com.gemwallet.android.data.service.store.database.AddressesDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemSwapper
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object TransactionModule {
    @Provides
    @Singleton
    fun provideGetTransactions(
        transactionRepository: TransactionRepository,
        addressDao: AddressesDao,
    ): GetTransactions {
        return GetTransactionsImpl(transactionRepository, addressDao)
    }
    @Provides
    @Singleton
    fun provideGetTransactionDetails(
        sessionRepository: SessionRepository,
        transactionRepository: TransactionRepository,
        assetsRepository: AssetsRepository,
        getCurrentBlockExplorer: GetCurrentBlockExplorer,
        gemSwapper: GemSwapper,
        addressDao: AddressesDao,
    ): GetTransactionDetails {
        return GetTransactionDetailsImpl(
            sessionRepository = sessionRepository,
            transactionRepository = transactionRepository,
            assetsRepository = assetsRepository,
            getCurrentBlockExplorer = getCurrentBlockExplorer,
            gemSwapper = gemSwapper,
            addressDao = addressDao,
        )
    }
}