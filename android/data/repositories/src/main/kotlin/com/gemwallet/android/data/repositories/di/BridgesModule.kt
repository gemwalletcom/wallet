package com.gemwallet.android.data.repositories.di

import com.gemwallet.android.data.repositories.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectClient
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequests
import com.gemwallet.android.data.repositories.bridge.WalletConnectRequestHandler
import com.gemwallet.android.data.repositories.gemstone.GemstoneConnectionStore
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
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
        walletsRepository: WalletsRepository,
        connectionsDao: ConnectionsDao,
    ): GemstoneConnectionStore = GemstoneConnectionStore(
        walletsRepository = walletsRepository,
        connectionsDao = connectionsDao,
    )

    @Singleton
    @Provides
    fun provideBridgeRepository(
        connectionStore: GemstoneConnectionStore,
        walletConnectClient: WalletConnectClient,
        walletConnectService: GemWalletConnectService,
    ): BridgesRepository = BridgesRepository(
        connectionStore = connectionStore,
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
