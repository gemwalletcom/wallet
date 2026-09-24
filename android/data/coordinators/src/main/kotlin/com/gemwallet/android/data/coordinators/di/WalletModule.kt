package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.data.coordinators.wallet.DeleteWalletImpl
import com.gemwallet.android.data.coordinators.wallet.GetAllWalletsImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletDetailsImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletImpl
import com.gemwallet.android.data.coordinators.wallet.GetWalletsImpl
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletModule {

    @Provides
    @Singleton
    fun provideGetWalletDetails(walletStore: GemstoneWalletStore, walletService: GemWalletService): GetWalletDetails = GetWalletDetailsImpl(walletStore, walletService)

    @Provides
    @Singleton
    fun provideGetWallets(walletStore: GemstoneWalletStore): GetWallets = GetWalletsImpl(walletStore)

    @Provides
    @Singleton
    fun provideGetWallet(walletStore: GemstoneWalletStore): GetWallet = GetWalletImpl(walletStore)

    @Provides
    @Singleton
    fun provideGetAllWallets(getSession: GetSession, walletStore: GemstoneWalletStore, walletService: GemWalletService): GetAllWallets = GetAllWalletsImpl(getSession, walletStore, walletService)

    @Provides
    fun provideDeleteWallet(walletService: GemWalletService): DeleteWallet = DeleteWalletImpl(walletService)
}
