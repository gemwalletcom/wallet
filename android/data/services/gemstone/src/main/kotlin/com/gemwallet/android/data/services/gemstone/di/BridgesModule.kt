package com.gemwallet.android.data.services.gemstone.di

import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.services.gemstone.stores.GemstoneConnectionStore
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSignMessageService
import uniffi.gemstone.GemSimulationService
import uniffi.gemstone.GemWalletConnectService
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletSessionService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BridgesModule {
    @Singleton
    @Provides
    fun provideGemstoneConnectionStore(walletStore: GemstoneWalletStore, connectionsDao: ConnectionsDao): GemstoneConnectionStore = GemstoneConnectionStore(
        walletStore = walletStore,
        connectionsDao = connectionsDao,
    )

    @Singleton
    @Provides
    fun provideWalletConnectPendingRequests(): WalletConnectPendingRequests = WalletConnectPendingRequests()

    @Singleton
    @Provides
    fun provideGemWalletConnectService(
        simulationService: GemSimulationService,
        connectionStore: GemstoneConnectionStore,
        pendingRequests: WalletConnectPendingRequests,
        walletSessionService: GemWalletSessionService,
        assetsService: GemAssetsService,
        signMessageService: GemSignMessageService,
    ): GemWalletConnectService = GemWalletConnectService(
        simulation = simulationService,
        store = connectionStore,
        signer = pendingRequests,
        session = walletSessionService,
        assets = assetsService,
        signMessage = signMessageService,
    )

    @Provides
    fun provideGemWalletConnectServiceInterface(service: GemWalletConnectService): GemWalletConnectServiceInterface = service
}
