package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectClient
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BridgesModule {
    @Singleton
    @Provides
    fun provideBridgeRepository(
        walletsRepository: WalletsRepository,
        connectionsDao: ConnectionsDao,
        walletConnectClient: WalletConnectClient,
    ): BridgesRepository = BridgesRepository(
        walletsRepository = walletsRepository,
        connectionsDao = connectionsDao,
        walletConnectClient = walletConnectClient,
    )
}
