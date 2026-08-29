package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.application.bridge.WalletConnectPendingRequests
import com.gemwallet.android.application.bridge.WalletConnectRequestHandler
import com.gemwallet.android.data.repositories.gemstone.GemstoneConnectionStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletStore
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletConnectService
import uniffi.gemstone.GemWalletSessionService
import uniffi.gemstone.TransactionSimulationService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BridgesModule {
    @Singleton
    @Provides
    fun provideGemstoneConnectionStore(
        walletStore: GemstoneWalletStore,
        connectionsDao: ConnectionsDao,
    ): GemstoneConnectionStore = GemstoneConnectionStore(
        walletStore = walletStore,
        connectionsDao = connectionsDao,
    )

    @Singleton
    @Provides
    fun provideWalletConnectPendingRequests(): WalletConnectPendingRequests = WalletConnectPendingRequests()

    @Singleton
    @Provides
    fun provideGemWalletConnectService(
        simulationService: TransactionSimulationService,
        connectionStore: GemstoneConnectionStore,
        pendingRequests: WalletConnectPendingRequests,
        walletSessionService: GemWalletSessionService,
    ): GemWalletConnectService = GemWalletConnectService(
        simulation = simulationService,
        store = connectionStore,
        signer = pendingRequests,
        session = walletSessionService,
    )

    @Singleton
    @Provides
    fun provideWalletConnectRequestHandler(
        service: GemWalletConnectService,
    ): WalletConnectRequestHandler = WalletConnectRequestHandler(service)
}
