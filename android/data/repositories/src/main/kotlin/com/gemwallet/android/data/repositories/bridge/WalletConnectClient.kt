package com.gemwallet.android.data.repositories.bridge

import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import kotlinx.coroutines.flow.Flow

interface WalletConnectClient {
    val isEnabled: Boolean
    val events: Flow<WalletConnectEvent>

    fun initialize(onSuccess: () -> Unit = {}, onError: (String) -> Unit = {})
    fun activeSessions(): List<WalletConnectSession>
    fun pendingSessionRequests(topic: String): List<WalletConnectSessionRequest>
    fun verifyContext(id: Long): WalletConnectVerifyContext?
    fun pingSession(topic: String)
    fun disconnectSession(topic: String, onSuccess: () -> Unit, onError: (String) -> Unit)
    fun pair(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit)
    fun generateApprovedNamespaces(
        proposal: WalletConnectSessionProposal,
        supportedNamespaces: Map<String, WalletConnectSessionNamespace>,
    ): Map<String, WalletConnectSessionNamespace>
    fun approveSession(
        proposal: WalletConnectSessionProposal,
        namespaces: Map<String, WalletConnectSessionNamespace>,
        properties: Map<String, String>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    )
    fun rejectSession(proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (String) -> Unit)
    fun approveAuthentication(
        request: WalletConnectAuthenticationRequest,
        auths: List<WalletConnectAuthObject>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    )
    fun rejectAuthentication(request: WalletConnectAuthenticationRequest, onSuccess: () -> Unit, onError: (String) -> Unit)
    fun respondSessionRequest(topic: String, id: Long, response: WalletConnectJsonRpcResponse, onSuccess: () -> Unit, onError: (String) -> Unit)
    fun generateAuthPayloadParams(
        payloadParams: WalletConnectAuthPayloadParams,
        supportedChains: List<String>,
        supportedMethods: List<String>,
    ): WalletConnectAuthPayloadParams
    fun formatAuthMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String
    fun generateAuthObject(payloadParams: WalletConnectAuthPayloadParams, issuer: String, signature: String): WalletConnectAuthObject
}

sealed interface WalletConnectEvent {
    data class SessionSettled(val session: WalletConnectSession) : WalletConnectEvent
    data class SessionDeleted(val topic: String) : WalletConnectEvent
    data class SessionProposal(val proposal: WalletConnectSessionProposal, val verifyContext: WalletConnectVerifyContext) : WalletConnectEvent
    data class SessionRequest(val request: WalletConnectSessionRequest, val verifyContext: WalletConnectVerifyContext) : WalletConnectEvent
    data class AuthenticationRequest(val request: WalletConnectAuthenticationRequest, val verifyContext: WalletConnectVerifyContext) : WalletConnectEvent
}

data class WalletConnectSession(
    val topic: String,
    val expiry: Long,
    val metadata: WalletConnectionSessionAppMetadata?,
    val namespaces: Map<String, WalletConnectSessionNamespace>,
    val redirect: String?,
)

data class WalletConnectSessionNamespace(
    val chains: List<String>?,
    val methods: List<String>,
    val events: List<String>,
    val accounts: List<String>,
)

data class WalletConnectProposalNamespace(
    val chains: List<String>?,
)

data class WalletConnectSessionProposal(
    val name: String,
    val description: String,
    val url: String,
    val icons: List<String>,
    val requiredNamespaces: Map<String, WalletConnectProposalNamespace>,
    val optionalNamespaces: Map<String, WalletConnectProposalNamespace>,
    val proposerPublicKey: String,
    val properties: Map<String, String>?,
)

data class WalletConnectJsonRpcRequest(
    val id: Long,
    val method: String,
    val params: String,
)

data class WalletConnectSessionRequest(
    val topic: String,
    val chainId: String?,
    val request: WalletConnectJsonRpcRequest,
)

data class WalletConnectAuthenticationRequest(
    val id: Long,
    val metadata: WalletConnectionSessionAppMetadata?,
    val payloadParams: WalletConnectAuthPayloadParams,
)

data class WalletConnectAuthPayloadParams(
    val chains: List<String>,
    val domain: String,
    val nonce: String,
    val aud: String,
    val type: String?,
    val iat: String,
    val nbf: String?,
    val exp: String?,
    val statement: String?,
    val requestId: String?,
    val resources: List<String>?,
    val signatureTypes: Map<String, List<String>>?,
)

interface WalletConnectAuthObject

data class WalletConnectVerifyContext(
    val origin: String,
    val validation: WalletConnectValidation,
    val isScam: Boolean?,
)

enum class WalletConnectValidation {
    Valid,
    Invalid,
    Unknown,
}

sealed interface WalletConnectJsonRpcResponse {
    data class Result(val payload: String) : WalletConnectJsonRpcResponse
    data class Error(val code: Int, val message: String) : WalletConnectJsonRpcResponse
}
