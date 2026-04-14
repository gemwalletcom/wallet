package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.confirm.coordinators.BuildConfirmProperties
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.application.confirm.coordinators.ValidateBalance
import com.gemwallet.android.blockchain.operators.LoadPrivateKeyOperator
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.SignClientProxy
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.coordinators.confirm.BuildConfirmPropertiesImpl
import com.gemwallet.android.data.coordinators.confirm.ConfirmTransactionImpl
import com.gemwallet.android.data.coordinators.confirm.ValidateBalanceImpl
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object ConfirmModule {

    @Provides
    @Singleton
    fun provideValidateBalance(): ValidateBalance = ValidateBalanceImpl()

    @Provides
    @Singleton
    fun provideConfirmTransaction(
        passwordStore: PasswordStore,
        loadPrivateKeyOperator: LoadPrivateKeyOperator,
        signClient: SignClientProxy,
        broadcastService: BroadcastService,
        createTransactionsCase: CreateTransaction,
        assetsRepository: AssetsRepository,
    ): ConfirmTransaction = ConfirmTransactionImpl(
        passwordStore,
        loadPrivateKeyOperator,
        signClient,
        broadcastService,
        createTransactionsCase,
        assetsRepository,
    )

    @Provides
    @Singleton
    fun provideBuildConfirmProperties(
        stakeRepository: StakeRepository,
        getCurrentBlockExplorer: GetCurrentBlockExplorer,
    ): BuildConfirmProperties = BuildConfirmPropertiesImpl(
        stakeRepository,
        getCurrentBlockExplorer,
    )
}
