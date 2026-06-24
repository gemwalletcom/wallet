package com.gemwallet.android.data.service.walletconnect.reown

import android.app.Application
import android.content.Context
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthObject
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthPayloadParams
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthenticationRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectClient
import com.gemwallet.android.data.repositories.bridge.WalletConnectEvent
import com.gemwallet.android.data.repositories.bridge.WalletConnectJsonRpcRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectJsonRpcResponse
import com.gemwallet.android.data.repositories.bridge.WalletConnectProposalNamespace
import com.gemwallet.android.data.repositories.bridge.WalletConnectSession
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionNamespace
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionProposal
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectValidation
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import com.gemwallet.android.ext.walletConnectIcon
import com.reown.android.Core
import com.reown.android.CoreClient
import com.reown.android.relay.ConnectionType
import com.reown.walletkit.client.Wallet
import com.reown.walletkit.client.WalletKit
import com.wallet.core.primitives.WalletConnectionSessionAppMetadata
import dagger.hilt.android.qualifiers.ApplicationContext
import java.net.URI
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ReownWalletConnectClient @Inject constructor(
    @ApplicationContext private val context: Context,
) : WalletConnectClient, WalletKit.WalletDelegate, CoreClient.CoreDelegate {

    override val isEnabled: Boolean = true

    private val walletEvents = MutableSharedFlow<WalletConnectEvent>(
        replay = 1,
        extraBufferCapacity = 64,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )
    override val events: SharedFlow<WalletConnectEvent> = walletEvents.asSharedFlow()

    override fun initialize(onSuccess: () -> Unit, onError: (String) -> Unit) {
        CoreClient.initialize(
            application = context as Application,
            projectId = PROJECT_ID,
            metaData = Core.Model.AppMetaData(
                name = "Gem Wallet",
                description = "Gem Web3 Wallet",
                url = "https://gemwallet.com",
                icons = listOf("https://gemwallet.com/images/gem-logo-256x256.png"),
                redirect = "gem://wc/",
            ),
            connectionType = ConnectionType.AUTOMATIC,
            telemetryEnabled = false,
        ) { error ->
            onError(error.throwable.message.orEmpty())
        }
        WalletKit.initialize(
            params = Wallet.Params.Init(core = CoreClient),
            onSuccess = {
                CoreClient.setDelegate(this)
                WalletKit.setWalletDelegate(this)
                onSuccess()
            },
            onError = { onError(it.throwable.message.orEmpty()) },
        )
    }

    override fun activeSessions(): List<WalletConnectSession> {
        return WalletKit.getListOfActiveSessions().map { it.toWalletConnectSession() }
    }

    override fun pendingSessionRequests(topic: String): List<WalletConnectSessionRequest> {
        return WalletKit.getPendingListOfSessionRequests(topic).map { it.toWalletConnectSessionRequest() }
    }

    override fun verifyContext(id: Long): WalletConnectVerifyContext? {
        return WalletKit.getVerifyContext(id)?.toWalletConnectVerifyContext()
    }

    override fun pingSession(topic: String) {
        WalletKit.pingSession(Wallet.Params.Ping(topic), null)
    }

    override fun disconnectSession(topic: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        WalletKit.disconnectSession(
            params = Wallet.Params.SessionDisconnect(topic),
            onSuccess = { onSuccess() },
            onError = { onError(it.throwable.message.orEmpty()) },
        )
    }

    override fun pair(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        WalletKit.pair(
            params = Wallet.Params.Pair(uri),
            onSuccess = { onSuccess() },
            onError = { onError(it.throwable.message.orEmpty()) },
        )
    }

    override fun generateApprovedNamespaces(
        proposal: WalletConnectSessionProposal,
        supportedNamespaces: Map<String, WalletConnectSessionNamespace>,
    ): Map<String, WalletConnectSessionNamespace> {
        val sessionProposal = proposal.pendingReownProposal() ?: return supportedNamespaces
        return WalletKit.generateApprovedNamespaces(
            sessionProposal = sessionProposal,
            supportedNamespaces = supportedNamespaces.mapValues { it.value.toReownSessionNamespace() },
        ).mapValues { it.value.toWalletConnectSessionNamespace() }
    }

    override fun approveSession(
        proposal: WalletConnectSessionProposal,
        namespaces: Map<String, WalletConnectSessionNamespace>,
        properties: Map<String, String>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val sessionProposal = proposal.pendingReownProposal()
        if (sessionProposal == null) {
            onError("WalletConnect session proposal is no longer available")
            return
        }
        WalletKit.approveSession(
            params = Wallet.Params.SessionApprove(
                proposerPublicKey = sessionProposal.proposerPublicKey,
                namespaces = namespaces.mapValues { it.value.toReownSessionNamespace() },
                properties = properties,
            ),
            onError = { onError(it.throwable.message ?: "Unknown error") },
            onSuccess = { onSuccess() },
        )
    }

    override fun rejectSession(proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (String) -> Unit) {
        val sessionProposal = proposal.pendingReownProposal()
        if (sessionProposal == null) {
            onSuccess()
            return
        }
        WalletKit.rejectSession(
            params = Wallet.Params.SessionReject(
                proposerPublicKey = sessionProposal.proposerPublicKey,
                reason = "Reject Session",
            ),
            onSuccess = { onSuccess() },
            onError = { onError(it.throwable.message.orEmpty()) },
        )
    }

    override fun approveAuthentication(
        request: WalletConnectAuthenticationRequest,
        auths: List<WalletConnectAuthObject>,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        WalletKit.approveSessionAuthenticate(
            params = Wallet.Params.ApproveSessionAuthenticate(
                id = request.id,
                auths = auths.map { (it as ReownAuthObject).value },
            ),
            onSuccess = { onSuccess() },
            onError = { onError(it.throwable.message ?: "Authentication failed") },
        )
    }

    override fun rejectAuthentication(request: WalletConnectAuthenticationRequest, onSuccess: () -> Unit, onError: (String) -> Unit) {
        WalletKit.rejectSessionAuthenticate(
            params = Wallet.Params.RejectSessionAuthenticate(
                id = request.id,
                reason = "Reject Session Authentication",
            ),
            onSuccess = { onSuccess() },
            onError = { onError(it.throwable.message.orEmpty()) },
        )
    }

    override fun respondSessionRequest(
        topic: String,
        id: Long,
        response: WalletConnectJsonRpcResponse,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        WalletKit.respondSessionRequest(
            params = Wallet.Params.SessionRequestResponse(
                sessionTopic = topic,
                jsonRpcResponse = when (response) {
                    is WalletConnectJsonRpcResponse.Result -> Wallet.Model.JsonRpcResponse.JsonRpcResult(id, response.payload)
                    is WalletConnectJsonRpcResponse.Error -> Wallet.Model.JsonRpcResponse.JsonRpcError(id, response.code, response.message)
                },
            ),
            onSuccess = { onSuccess() },
            onError = { onError(it.throwable.message.orEmpty()) },
        )
    }

    override fun generateAuthPayloadParams(
        payloadParams: WalletConnectAuthPayloadParams,
        supportedChains: List<String>,
        supportedMethods: List<String>,
    ): WalletConnectAuthPayloadParams {
        val result = WalletKit.generateAuthPayloadParams(
            payloadParams = payloadParams.toReownPayloadAuthRequestParams(),
            supportedChains = supportedChains,
            supportedMethods = supportedMethods,
        )
        return result.toWalletConnectAuthPayloadParams()
    }

    override fun formatAuthMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String {
        return WalletKit.formatAuthMessage(
            Wallet.Params.FormatAuthMessage(
                payloadParams = payloadParams.toReownPayloadAuthRequestParams(),
                issuer = issuer,
            )
        )
    }

    override fun generateAuthObject(
        payloadParams: WalletConnectAuthPayloadParams,
        issuer: String,
        signature: String,
    ): WalletConnectAuthObject {
        return ReownAuthObject(
            WalletKit.generateAuthObject(
                payloadParams = payloadParams.toReownPayloadAuthRequestParams(),
                issuer = issuer,
                signature = Wallet.Model.Cacao.Signature(
                    t = "eip191",
                    s = signature,
                ),
            )
        )
    }

    override val onSessionAuthenticate: (Wallet.Model.SessionAuthenticate, Wallet.Model.VerifyContext) -> Unit = { request, verifyContext ->
        walletEvents.tryEmit(WalletConnectEvent.AuthenticationRequest(request.toWalletConnectAuthenticationRequest(), verifyContext.toWalletConnectVerifyContext()))
    }

    override fun onConnectionStateChange(state: Wallet.Model.ConnectionState) = Unit
    override fun onError(error: Wallet.Model.Error) = Unit
    override fun onProposalExpired(proposal: Wallet.Model.ExpiredProposal) = Unit
    override fun onRequestExpired(request: Wallet.Model.ExpiredRequest) = Unit

    override fun onSessionDelete(sessionDelete: Wallet.Model.SessionDelete) {
        if (sessionDelete is Wallet.Model.SessionDelete.Success) {
            walletEvents.tryEmit(WalletConnectEvent.SessionDeleted(sessionDelete.topic))
        }
    }

    override fun onSessionExtend(session: Wallet.Model.Session) = Unit

    override fun onSessionProposal(sessionProposal: Wallet.Model.SessionProposal, verifyContext: Wallet.Model.VerifyContext) {
        walletEvents.tryEmit(WalletConnectEvent.SessionProposal(sessionProposal.toWalletConnectSessionProposal(), verifyContext.toWalletConnectVerifyContext()))
    }

    override fun onSessionRequest(sessionRequest: Wallet.Model.SessionRequest, verifyContext: Wallet.Model.VerifyContext) {
        walletEvents.tryEmit(WalletConnectEvent.SessionRequest(sessionRequest.toWalletConnectSessionRequest(), verifyContext.toWalletConnectVerifyContext()))
    }

    override fun onSessionSettleResponse(settleSessionResponse: Wallet.Model.SettledSessionResponse) {
        if (settleSessionResponse is Wallet.Model.SettledSessionResponse.Result) {
            walletEvents.tryEmit(WalletConnectEvent.SessionSettled(settleSessionResponse.session.toWalletConnectSession()))
        }
    }

    override fun onSessionUpdateResponse(sessionUpdateResponse: Wallet.Model.SessionUpdateResponse) = Unit

    private fun WalletConnectSessionProposal.pendingReownProposal(): Wallet.Model.SessionProposal? {
        return WalletKit.getSessionProposals().firstOrNull { it.proposerPublicKey == proposerPublicKey }
    }

    private data class ReownAuthObject(val value: Wallet.Model.Cacao) : WalletConnectAuthObject

    private companion object {
        const val PROJECT_ID = "3bc07cd7179d11ea65335fb9377702b6"
    }
}

private fun Wallet.Model.Session.toWalletConnectSession(): WalletConnectSession {
    return WalletConnectSession(
        topic = topic,
        expiry = expiry,
        metadata = metaData?.toWalletConnectionSessionAppMetadata(),
        namespaces = namespaces.mapValues { it.value.toWalletConnectSessionNamespace() },
        redirect = redirect,
    )
}

private fun Core.Model.AppMetaData.toWalletConnectionSessionAppMetadata(): WalletConnectionSessionAppMetadata {
    return WalletConnectionSessionAppMetadata(
        name = name,
        description = description,
        url = url,
        icon = icons.walletConnectIcon(),
    )
}

private fun Wallet.Model.Namespace.Session.toWalletConnectSessionNamespace(): WalletConnectSessionNamespace {
    return WalletConnectSessionNamespace(
        chains = chains,
        methods = methods,
        events = events,
        accounts = accounts,
    )
}

private fun WalletConnectSessionNamespace.toReownSessionNamespace(): Wallet.Model.Namespace.Session {
    return Wallet.Model.Namespace.Session(
        chains = chains,
        accounts = accounts,
        methods = methods,
        events = events,
    )
}

private fun Wallet.Model.Namespace.Proposal.toWalletConnectProposalNamespace(): WalletConnectProposalNamespace {
    return WalletConnectProposalNamespace(chains = chains)
}

private fun Wallet.Model.SessionProposal.toWalletConnectSessionProposal(): WalletConnectSessionProposal {
    return WalletConnectSessionProposal(
        name = name,
        description = description,
        url = url,
        icons = icons.map(URI::toString),
        requiredNamespaces = requiredNamespaces.mapValues { it.value.toWalletConnectProposalNamespace() },
        optionalNamespaces = optionalNamespaces.mapValues { it.value.toWalletConnectProposalNamespace() },
        proposerPublicKey = proposerPublicKey,
        properties = properties,
    )
}

private fun Wallet.Model.SessionRequest.toWalletConnectSessionRequest(): WalletConnectSessionRequest {
    return WalletConnectSessionRequest(
        topic = topic,
        chainId = chainId,
        request = WalletConnectJsonRpcRequest(
            id = request.id,
            method = request.method,
            params = request.params,
        ),
    )
}

private fun Wallet.Model.SessionAuthenticate.toWalletConnectAuthenticationRequest(): WalletConnectAuthenticationRequest {
    return WalletConnectAuthenticationRequest(
        id = id,
        metadata = participant.metadata?.toWalletConnectionSessionAppMetadata(),
        payloadParams = payloadParams.toWalletConnectAuthPayloadParams(),
    )
}

private fun Wallet.Model.PayloadAuthRequestParams.toWalletConnectAuthPayloadParams(): WalletConnectAuthPayloadParams {
    return WalletConnectAuthPayloadParams(
        chains = chains,
        domain = domain,
        nonce = nonce,
        aud = aud,
        type = type,
        iat = iat,
        nbf = nbf,
        exp = exp,
        statement = statement,
        requestId = requestId,
        resources = resources,
        signatureTypes = signatureTypes,
    )
}

private fun WalletConnectAuthPayloadParams.toReownPayloadAuthRequestParams(): Wallet.Model.PayloadAuthRequestParams {
    return Wallet.Model.PayloadAuthRequestParams(
        chains = chains,
        domain = domain,
        nonce = nonce,
        aud = aud,
        type = type,
        iat = iat,
        nbf = nbf,
        exp = exp,
        statement = statement,
        requestId = requestId,
        resources = resources,
        signatureTypes = signatureTypes,
    )
}

private fun Wallet.Model.VerifyContext.toWalletConnectVerifyContext(): WalletConnectVerifyContext {
    return WalletConnectVerifyContext(
        origin = origin,
        validation = when (validation) {
            Wallet.Model.Validation.VALID -> WalletConnectValidation.Valid
            Wallet.Model.Validation.INVALID -> WalletConnectValidation.Invalid
            Wallet.Model.Validation.UNKNOWN -> WalletConnectValidation.Unknown
        },
        isScam = isScam,
    )
}
