package com.gemwallet.android.data.coordinators.di

import uniffi.gemstone.GemExplorerService
import com.gemwallet.android.application.confirm.cases.BuildConfirmProperties
import com.gemwallet.android.application.confirm.cases.ConfirmTransaction
import com.gemwallet.android.application.confirm.cases.CalculateTransferAmount
import com.gemwallet.android.application.confirm.cases.GetFeeAssets
import uniffi.gemstone.GemConfirmServiceInterface
import uniffi.gemstone.GemTransactionSigner
import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.data.coordinators.confirm.BuildConfirmPropertiesImpl
import com.gemwallet.android.data.coordinators.confirm.ConfirmTransactionImpl
import com.gemwallet.android.data.coordinators.confirm.CalculateTransferAmountImpl
import com.gemwallet.android.data.coordinators.confirm.GetFeeAssetsImpl
import com.gemwallet.android.data.coordinators.confirm.ChainFeeAssetProvider
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.wallet.core.primitives.Chain
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.data.services.gemstone.stores.GemstoneAssetStore
import com.gemwallet.android.application.session.cases.GetCurrentWalletId

@InstallIn(SingletonComponent::class)
@Module
object ConfirmModule {

    @Provides
    @Singleton
    fun provideCalculateTransferAmount(): CalculateTransferAmount = CalculateTransferAmountImpl()

    @Provides
    @Singleton
    fun provideGetFeeAssets(assetStore: GemstoneAssetStore, getCurrentWalletId: GetCurrentWalletId): GetFeeAssets = GetFeeAssetsImpl(
        providers = mapOf(Chain.Tempo to ChainFeeAssetProvider(Chain.Tempo, assetStore, getCurrentWalletId)),
    )

    @Provides
    @Singleton
    fun provideConfirmTransaction(
        signer: GemTransactionSigner,
        confirmService: GemConfirmServiceInterface,
        createTransactionsCase: CreateTransaction,
        recentAssetsService: RecentAssetsService,
    ): ConfirmTransaction = ConfirmTransactionImpl(
        signer,
        confirmService,
        createTransactionsCase,
        recentAssetsService,
    )

    @Provides
    @Singleton
    fun provideBuildConfirmProperties(
        getStakeValidator: GetStakeValidator,
        explorerService: GemExplorerService,
    ): BuildConfirmProperties = BuildConfirmPropertiesImpl(
        getStakeValidator,
        explorerService,
    )
}
