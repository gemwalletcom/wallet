package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.confirm.coordinators.BuildConfirmProperties
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.application.confirm.coordinators.CalculateTransferAmount
import com.gemwallet.android.application.confirm.coordinators.GetFeeAssets
import uniffi.gemstone.GemConfirmServiceInterface
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.coordinators.confirm.BuildConfirmPropertiesImpl
import com.gemwallet.android.data.coordinators.confirm.ConfirmTransactionImpl
import com.gemwallet.android.data.coordinators.confirm.CalculateTransferAmountImpl
import com.gemwallet.android.data.coordinators.confirm.GetFeeAssetsImpl
import com.gemwallet.android.data.coordinators.confirm.TempoFeeAssetProvider
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.assets.RecentAssetsService
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.wallet.core.primitives.Chain
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
    fun provideCalculateTransferAmount(): CalculateTransferAmount = CalculateTransferAmountImpl()

    @Provides
    @Singleton
    fun provideGetFeeAssets(assetsRepository: AssetsRepository): GetFeeAssets = GetFeeAssetsImpl(
        providers = mapOf(Chain.Tempo to TempoFeeAssetProvider(assetsRepository)),
    )

    @Provides
    @Singleton
    fun provideConfirmTransaction(
        passwordStore: PasswordStore,
        signTransactionOperator: GemSignTransactionOperator,
        confirmService: GemConfirmServiceInterface,
        createTransactionsCase: CreateTransaction,
        recentAssetsService: RecentAssetsService,
    ): ConfirmTransaction = ConfirmTransactionImpl(
        passwordStore,
        signTransactionOperator,
        confirmService,
        createTransactionsCase,
        recentAssetsService,
    )

    @Provides
    @Singleton
    fun provideBuildConfirmProperties(
        stakeRepository: StakeRepository,
        explorerService: GemExplorerService,
    ): BuildConfirmProperties = BuildConfirmPropertiesImpl(
        stakeRepository,
        explorerService,
    )
}
