package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemTransactionDetailsService
import uniffi.gemstone.GemAddressService
import com.gemwallet.android.application.transactions.cases.GetTransactionDetails
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.data.coordinators.transaction.GetTransactionDetailsImpl
import com.gemwallet.android.data.coordinators.transaction.GetTransactionsImpl
import com.gemwallet.android.application.transactions.cases.GetTransaction
import com.gemwallet.android.application.transactions.cases.GetPendingTransactionsCount
import com.gemwallet.android.data.coordinators.transaction.GetPendingTransactionsCountImpl
import com.gemwallet.android.data.coordinators.transaction.GetTransactionImpl
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
import uniffi.gemstone.GemAssetConfigService
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.session.cases.GetCurrentWalletId

@InstallIn(SingletonComponent::class)
@Module
object TransactionModule {
    @Provides
    @Singleton
    fun provideGetTransactions(
        getCurrentWalletId: GetCurrentWalletId,
        transactionStore: GemstoneTransactionStore,
        addressService: GemAddressService,
    ): GetTransactions = GetTransactionsImpl(getCurrentWalletId, transactionStore, addressService)

    @Provides
    @Singleton
    fun provideGetTransaction(
        getCurrentWalletId: GetCurrentWalletId,
        transactionStore: GemstoneTransactionStore,
    ): GetTransaction = GetTransactionImpl(getCurrentWalletId, transactionStore)

    @Provides
    @Singleton
    fun provideGetPendingTransactionsCount(
        getCurrentWalletId: GetCurrentWalletId,
        transactionStore: GemstoneTransactionStore,
        assetConfig: GemAssetConfigService,
    ): GetPendingTransactionsCount = GetPendingTransactionsCountImpl(getCurrentWalletId, transactionStore, assetConfig)


    @Provides
    @Singleton
    fun provideGetTransactionDetails(
        getSession: GetSession,
        getTransaction: GetTransaction,
        transactionDetailsService: GemTransactionDetailsService,
    ): GetTransactionDetails {
        return GetTransactionDetailsImpl(
            getSession = getSession,
            getTransaction = getTransaction,
            transactionDetailsService = transactionDetailsService,
        )
    }
}
