package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_import.cases.SetupWallet
import com.gemwallet.android.data.coordinators.wallet_import.SetupWalletImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAppStartService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletImportModule {


    @Provides
    @Singleton
    fun provideSetupWallet(
        appStartService: GemAppStartService,
    ): SetupWallet = SetupWalletImpl(appStartService)
}
