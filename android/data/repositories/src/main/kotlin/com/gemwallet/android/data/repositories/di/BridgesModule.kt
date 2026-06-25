package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.ConnectionsRepository
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
    fun provideConnectionsRepository(
        walletsRepository: WalletsRepository,
        connectionsDao: ConnectionsDao,
    ): ConnectionsRepository = ConnectionsRepository(
        walletsRepository = walletsRepository,
        connectionsDao = connectionsDao,
    )

    @Singleton
    @Provides
    fun provideBridgeRepository(
        connectionsRepository: ConnectionsRepository,
        walletConnectClient: WalletConnectClient,
    ): BridgesRepository = BridgesRepository(
        connectionsRepository = connectionsRepository,
        walletConnectClient = walletConnectClient,
    )
}
