package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_connect.coordinators.PrepareSessionProposal
import com.gemwallet.android.data.coordinators.wallet_connect.PrepareSessionProposalImpl
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletConnectService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletConnectModule {

    @Provides
    @Singleton
    fun providePrepareSessionProposal(
        walletConnectService: GemWalletConnectService,
    ): PrepareSessionProposal = PrepareSessionProposalImpl(
        walletConnectService = walletConnectService,
    )
}
