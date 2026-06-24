package com.gemwallet.android.data.service.walletconnect.noop

import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthObject
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthPayloadParams
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthenticationRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectClient
import com.gemwallet.android.data.repositories.bridge.WalletConnectEvent
import com.gemwallet.android.data.repositories.bridge.WalletConnectJsonRpcResponse
import com.gemwallet.android.data.repositories.bridge.WalletConnectSession
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionNamespace
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionProposal
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class NoopWalletConnectClient @Inject constructor() : WalletConnectClient {
    override val isEnabled: Boolean = false
    override val events: Flow<WalletConnectEvent> = emptyFlow()

    override fun initialize(onSuccess: () -> Unit, onError: (String) -> Unit) = onSuccess()
    override fun activeSessions(): List<WalletConnectSession> = emptyList()
    override fun pendingSessionRequests(topic: String): List<WalletConnectSessionRequest> = emptyList()
    override fun verifyContext(id: Long): WalletConnectVerifyContext? = null
    override fun pingSession(topic: String) = Unit
    override fun disconnectSession(topic: String, onSuccess: () -> Unit, onError: (String) -> Unit) = onSuccess()
    override fun pair(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit) = onError(UNAVAILABLE)
    override fun generateApprovedNamespaces(
        proposal: WalletConnectSessionProposal,
        supportedNamespaces: Map<String, WalletConnectSessionNamespace>,
    ): Map<String, WalletConnectSessionNamespace> = supportedNamespaces

    override fun approveSession(
        proposal: WalletConnectSessionProposal,
        namespaces: Map<String, WalletConnectSessionNamespace>,
        properties: Map<String, String>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) = onError(UNAVAILABLE)

    override fun rejectSession(proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (String) -> Unit) = onSuccess()

    override fun approveAuthentication(
        request: WalletConnectAuthenticationRequest,
        auths: List<WalletConnectAuthObject>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) = onError(UNAVAILABLE)

    override fun rejectAuthentication(request: WalletConnectAuthenticationRequest, onSuccess: () -> Unit, onError: (String) -> Unit) = onSuccess()

    override fun respondSessionRequest(
        topic: String,
        id: Long,
        response: WalletConnectJsonRpcResponse,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) = onSuccess()

    override fun generateAuthPayloadParams(
        payloadParams: WalletConnectAuthPayloadParams,
        supportedChains: List<String>,
        supportedMethods: List<String>,
    ): WalletConnectAuthPayloadParams = payloadParams

    override fun formatAuthMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String = ""

    override fun generateAuthObject(
        payloadParams: WalletConnectAuthPayloadParams,
        issuer: String,
        signature: String,
    ): WalletConnectAuthObject = NoopAuthObject

    private object NoopAuthObject : WalletConnectAuthObject

    private companion object {
        const val UNAVAILABLE = "WalletConnect is not available in this build"
    }
}
