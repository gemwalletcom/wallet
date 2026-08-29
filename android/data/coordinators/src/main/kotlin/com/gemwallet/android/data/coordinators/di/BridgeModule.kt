package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.application.bridge.WalletConnectClient
import com.gemwallet.android.application.bridge.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.application.bridge.cases.ApproveWalletConnection
import com.gemwallet.android.application.bridge.cases.DisconnectWalletConnection
import com.gemwallet.android.application.bridge.cases.GetWalletConnections
import com.gemwallet.android.application.bridge.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.bridge.cases.PairWalletConnect
import com.gemwallet.android.application.bridge.cases.RespondWalletConnectRequest
import com.gemwallet.android.data.coordinators.bridge.WalletConnectCoordinator
import com.gemwallet.android.data.repositories.gemstone.GemstoneConnectionStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import uniffi.gemstone.GemWalletConnectService
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object BridgeModule {
    @Singleton
    @Provides
    fun provideWalletConnectCoordinator(
        connectionStore: GemstoneConnectionStore,
        walletConnectClient: WalletConnectClient,
        walletConnectService: GemWalletConnectService,
    ): WalletConnectCoordinator = WalletConnectCoordinator(
        connectionStore = connectionStore,
        walletConnectClient = walletConnectClient,
        walletConnectService = walletConnectService,
    )

    @Singleton
    @Provides
    fun provideActiveWalletConnectRequest(
        coordinator: WalletConnectCoordinator,
    ): ActiveWalletConnectRequest = ActiveWalletConnectRequest(events = coordinator.bridgeEvents)

    @Provides
    fun provideIsWalletConnectEnabled(coordinator: WalletConnectCoordinator): IsWalletConnectEnabled = coordinator

    @Provides
    fun providePairWalletConnect(coordinator: WalletConnectCoordinator): PairWalletConnect = coordinator

    @Provides
    fun provideGetWalletConnections(coordinator: WalletConnectCoordinator): GetWalletConnections = coordinator

    @Provides
    fun provideDisconnectWalletConnection(coordinator: WalletConnectCoordinator): DisconnectWalletConnection = coordinator

    @Provides
    fun provideApproveWalletConnection(coordinator: WalletConnectCoordinator): ApproveWalletConnection = coordinator

    @Provides
    fun provideApproveWalletConnectAuthentication(
        coordinator: WalletConnectCoordinator,
    ): ApproveWalletConnectAuthentication = coordinator

    @Provides
    fun provideRespondWalletConnectRequest(coordinator: WalletConnectCoordinator): RespondWalletConnectRequest = coordinator
}
