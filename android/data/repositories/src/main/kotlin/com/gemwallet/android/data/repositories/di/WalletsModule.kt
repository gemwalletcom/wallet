package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.wallet.coordinators.WalletIdGenerator
import com.gemwallet.android.blockchain.operators.CreateAccountOperator
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepositoryImpl
import com.gemwallet.android.data.service.store.database.AccountsDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.WalletsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent

@InstallIn(SingletonComponent::class)
@Module
object WalletsModule {

    @Provides
    fun provideWalletsRepository(
        walletsDao: WalletsDao,
        accountsDao: AccountsDao,
        assetsDao: AssetsDao,
        createAccountOperator: CreateAccountOperator,
        walletIdGenerator: WalletIdGenerator,
        transactionRunner: StoreTransactionRunner,
    ): WalletsRepository {
        return WalletsRepositoryImpl(
            walletsDao = walletsDao,
            accountsDao = accountsDao,
            assetsDao = assetsDao,
            createAccount = createAccountOperator,
            walletIdGenerator = walletIdGenerator,
            transactionRunner = transactionRunner,
        )
    }

}
