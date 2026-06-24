package com.gemwallet.android.data.service.walletconnect.reown

import com.gemwallet.android.data.repositories.bridge.WalletConnectClient
import dagger.Binds
import dagger.Module
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
abstract class WalletConnectClientModule {
    @Binds
    @Singleton
    abstract fun bindWalletConnectClient(client: ReownWalletConnectClient): WalletConnectClient
}
