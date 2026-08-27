package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.ConnectionsRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectClient
import com.gemwallet.android.data.repositories.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequests
import com.gemwallet.android.data.repositories.bridge.WalletConnectRequestHandler
import com.gemwallet.android.data.repositories.gemstone.GemstoneConnectionStore
import com.gemwallet.android.data.repositories.gemstone.GemstoneWalletConnectSigner
import uniffi.gemstone.GemWalletConnectService
import uniffi.gemstone.TransactionSimulationService

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
        walletConnectService: GemWalletConnectService,
    ): BridgesRepository = BridgesRepository(
        connectionsRepository = connectionsRepository,
        walletConnectClient = walletConnectClient,
        walletConnectService = walletConnectService,
    )

    @Singleton
    @Provides
    fun provideActiveWalletConnectRequest(
        bridgesRepository: BridgesRepository,
    ): ActiveWalletConnectRequest = ActiveWalletConnectRequest(
        events = bridgesRepository.bridgeEvents,
    )

    @Singleton
    @Provides
    fun provideWalletConnectPendingRequests(): WalletConnectPendingRequests = WalletConnectPendingRequests()

    @Singleton
    @Provides
    fun provideGemWalletConnectService(
        simulationService: TransactionSimulationService,
        connectionsRepository: ConnectionsRepository,
        pendingRequests: WalletConnectPendingRequests,
    ): GemWalletConnectService = GemWalletConnectService(
        simulation = simulationService,
        store = GemstoneConnectionStore(connectionsRepository),
        signer = GemstoneWalletConnectSigner(pendingRequests),
    )

    @Singleton
    @Provides
    fun provideWalletConnectRequestHandler(
        service: GemWalletConnectService,
    ): WalletConnectRequestHandler = WalletConnectRequestHandler(service)
}
