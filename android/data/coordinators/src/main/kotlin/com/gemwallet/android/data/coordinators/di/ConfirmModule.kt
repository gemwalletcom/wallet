package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.confirm.coordinators.BuildConfirmProperties
import com.gemwallet.android.application.confirm.coordinators.ConfirmTransaction
import com.gemwallet.android.application.confirm.coordinators.CalculateTransferAmount
import com.gemwallet.android.application.confirm.coordinators.GetFeeAssets
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.GemSignTransactionOperator
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
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
        broadcastService: BroadcastService,
        createTransactionsCase: CreateTransaction,
        recentAssetsService: RecentAssetsService,
    ): ConfirmTransaction = ConfirmTransactionImpl(
        passwordStore,
        signTransactionOperator,
        broadcastService,
        createTransactionsCase,
        recentAssetsService,
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
