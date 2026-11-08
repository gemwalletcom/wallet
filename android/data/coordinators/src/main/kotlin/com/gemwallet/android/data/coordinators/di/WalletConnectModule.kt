package com.gemwallet.android.data.coordinators.di

import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.SyncWalletConnectSessions
import com.gemwallet.android.data.coordinators.wallet_connect.WalletConnectCoordinator
import com.gemwallet.android.data.coordinators.wallet_connect.WalletConnectListener
import com.gemwallet.android.data.coordinators.wallet_connect.WalletConnectRelay
import com.gemwallet.android.data.services.gemstone.stream.WebSocketConnection
import com.gemwallet.android.data.services.gemstone.stream.WebSocketRequest
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import uniffi.gemstone.AlienProvider
import uniffi.gemstone.GemConnectionService
import uniffi.gemstone.GemSecureStore
import uniffi.gemstone.GemWalletConnectClient
import uniffi.gemstone.GemWalletConnectService
import uniffi.gemstone.Platform
import uniffi.gemstone.walletConnectRelay
import javax.inject.Singleton

@InstallIn(SingletonComponent::class)
@Module
object WalletConnectModule {
    @Singleton
    @Provides
    fun provideWalletConnectRelay(secureStore: GemSecureStore, okHttpClient: OkHttpClient, connectionService: GemConnectionService): WalletConnectRelay = WalletConnectRelay(
        WebSocketConnection(
            requestProvider = {
                withContext(Dispatchers.IO) {
                    walletConnectRelay(secureStore, Platform.ANDROID).let { WebSocketRequest(url = it.url, headers = mapOf("Origin" to it.origin)) }
                }
            },
            client = okHttpClient,
            connectionService = connectionService,
        ),
    )

    @Singleton
    @Provides
    fun provideWalletConnectListener(): WalletConnectListener = WalletConnectListener()

    @Singleton
    @Provides
    fun provideGemWalletConnectClient(walletConnectService: GemWalletConnectService, secureStore: GemSecureStore, alienProvider: AlienProvider, relay: WalletConnectRelay, listener: WalletConnectListener): GemWalletConnectClient =
        GemWalletConnectClient(
            service = walletConnectService,
            store = secureStore,
            provider = alienProvider,
            transport = relay,
            listener = listener,
            platform = Platform.ANDROID,
        )

    @Singleton
    @Provides
    fun provideWalletConnectCoordinator(client: GemWalletConnectClient, relay: WalletConnectRelay, walletConnectService: GemWalletConnectService): WalletConnectCoordinator = WalletConnectCoordinator(
        client = client,
        relay = relay,
        walletConnectService = walletConnectService,
    )

    @Singleton
    @Provides
    fun provideActiveWalletConnectRequest(listener: WalletConnectListener, pendingRequests: WalletConnectPendingRequests): ActiveWalletConnectRequest = ActiveWalletConnectRequest(
        events = listener.events,
        pendingRequests = pendingRequests,
    )

    @Provides
    fun providePairWalletConnect(coordinator: WalletConnectCoordinator): PairWalletConnect = coordinator

    @Provides
    fun provideSyncWalletConnectSessions(coordinator: WalletConnectCoordinator): SyncWalletConnectSessions = coordinator

    @Provides
    fun provideDisconnectWalletConnection(coordinator: WalletConnectCoordinator): DisconnectWalletConnection = coordinator

    @Provides
    fun provideApproveWalletConnection(coordinator: WalletConnectCoordinator): ApproveWalletConnection = coordinator

    @Provides
    fun provideApproveWalletConnectAuthentication(coordinator: WalletConnectCoordinator): ApproveWalletConnectAuthentication = coordinator

    @Provides
    fun provideRespondWalletConnectRequest(coordinator: WalletConnectCoordinator): RespondWalletConnectRequest = coordinator
}
