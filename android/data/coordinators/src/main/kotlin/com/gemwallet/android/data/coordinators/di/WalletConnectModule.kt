package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_connect.coordinators.PrepareSessionProposal
import com.gemwallet.android.data.coordinators.wallet_connect.PrepareSessionProposalImpl
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
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
        sessionRepository: SessionRepository,
        walletsRepository: WalletsRepository,
        walletConnectService: GemWalletConnectService,
    ): PrepareSessionProposal = PrepareSessionProposalImpl(
        sessionRepository = sessionRepository,
        walletsRepository = walletsRepository,
        walletConnectService = walletConnectService,
    )
}
