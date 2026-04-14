package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.assets.coordinators.EnsureWalletAssets
import com.gemwallet.android.application.assets.coordinators.PrefetchAssets
import com.gemwallet.android.application.transactions.coordinators.GetTransactionDetails
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.cases.transactions.SaveTransactions
import com.gemwallet.android.data.coordinators.transaction.GetTransactionDetailsImpl
import com.gemwallet.android.data.coordinators.transaction.GetTransactionsImpl
import com.gemwallet.android.data.coordinators.transaction.SyncTransactionsImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.transactions.TransactionRepository
import com.gemwallet.android.data.service.store.WalletPreferencesFactory
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
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
    ): GetTransactions {
        return GetTransactionsImpl(transactionRepository)
    }
    @Provides
    @Singleton
    fun provideSyncTransactions(
        walletPreferencesFactory: WalletPreferencesFactory,
        gemDeviceApiClient: GemDeviceApiClient,
        saveTransactions: SaveTransactions,
        prefetchAssets: PrefetchAssets,
        ensureWalletAssets: EnsureWalletAssets,
    ): SyncTransactions {
        return SyncTransactionsImpl(
            walletPreferencesFactory = walletPreferencesFactory,
            gemDeviceApiClient = gemDeviceApiClient,
            saveTransactions = saveTransactions,
            prefetchAssets = prefetchAssets,
            ensureWalletAssets = ensureWalletAssets,
        )
    }

    @Provides
    @Singleton
    fun provideGetTransactionDetails(
        sessionRepository: SessionRepository,
        transactionRepository: TransactionRepository,
        assetsRepository: AssetsRepository,
        getCurrentBlockExplorer: GetCurrentBlockExplorer,
        gemSwapper: GemSwapper,
    ): GetTransactionDetails {
        return GetTransactionDetailsImpl(
            sessionRepository = sessionRepository,
            transactionRepository = transactionRepository,
            assetsRepository = assetsRepository,
            getCurrentBlockExplorer = getCurrentBlockExplorer,
            gemSwapper = gemSwapper,
        )
    }
}
