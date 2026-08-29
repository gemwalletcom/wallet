package com.gemwallet.android.data.coordinators.wallet_connect

import android.util.Log
import androidx.core.net.toUri
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthObject
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthPayloadParams
import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectClient
import com.gemwallet.android.application.wallet_connect.WalletConnectEvent
import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.WalletConnectSession
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.GetWalletConnections
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.toConnectionSession
import com.gemwallet.android.application.wallet_connect.toSupportedNamespaces
import com.gemwallet.android.data.services.gemstone.stores.GemstoneConnectionStore
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletConnectServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectCoordinator(
    private val connectionStore: GemstoneConnectionStore,
    private val walletConnectClient: WalletConnectClient,
    private val walletConnectService: GemWalletConnectServiceInterface,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
) : IsWalletConnectEnabled,
    PairWalletConnect,
    GetWalletConnections,
    DisconnectWalletConnection,
    ApproveWalletConnection,
    ApproveWalletConnectAuthentication,
    RespondWalletConnectRequest {

    private val pendingEvents = MutableSharedFlow<WalletConnectEvent>(extraBufferCapacity = 16)
    private val isWalletConnectInit = MutableStateFlow(false)
    val bridgeEvents = isWalletConnectInit.flatMapLatest {
        if (it) {
            merge(walletConnectClient.events, pendingEvents)
        } else {
            emptyFlow()
        }
    }

    init {
        scope.launch(Dispatchers.IO) {
            if (walletConnectService.hasSessions()) {
                initWalletConnect()
                sync()
                pingActiveSessions()
                handlePendingRequests()
            }
        }
        scope.launch(Dispatchers.IO) {
            bridgeEvents.collect { event ->
                when (event) {
                    is WalletConnectEvent.SessionDeleted -> walletConnectService.deleteSession(event.topic)
                    else -> Unit
                }
            }
        }
    }

    override fun isWalletConnectEnabled(): Boolean = walletConnectClient.isEnabled

    override fun observeConnections(): Flow<List<WalletConnection>> = connectionStore.observeConnections()

    override fun observeConnection(connectionId: String): Flow<WalletConnection?> = connectionStore.observeConnection(connectionId)

    override suspend fun getConnectionByTopic(topic: String): WalletConnection? = connectionStore.getConnectionBySessionId(topic)

    override suspend fun disconnect(connectionId: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        walletConnectService.deleteSession(connectionId)
        val activeSession = activeSessions()?.firstOrNull { it.topic == connectionId }
        if (activeSession != null) {
            walletConnectClient.disconnectSession(activeSession.topic, onSuccess = {}, onError = {})
        }
        onSuccess()
    }

    override fun pair(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        initWalletConnect(
            onSuccess = {
                try {
                    walletConnectClient.pair(
                        uri = uri,
                        onSuccess = { onSuccess() },
                        onError = { onError(it.ifBlank { "Pair to ${uri.toUri().host} fail" }) },
                    )
                } catch (err: Throwable) {
                    onError("Wallet Connect unavailable: ${err.message}")
                }
            },
            onError = { onError(it.ifBlank { "Wallet Connect unavailable" }) },
        )
    }

    override fun approveConnection(
        wallet: Wallet,
        proposal: WalletConnectSessionProposal,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        val approval = walletConnectService.sessionApproval(wallet = wallet.toJson())
        val sessionNamespaces = walletConnectClient.generateApprovedNamespaces(
            proposal = proposal,
            supportedNamespaces = approval.toSupportedNamespaces(),
        )
        val sessionProperties = walletConnectService.configSessionProperties(
            properties = proposal.properties ?: emptyMap(),
            caip2Chains = sessionNamespaces.values.flatMap { it.chains.orEmpty() },
            accounts = approval.accounts,
        )
        approveAndStoreSession(wallet, "Connection failed", onSuccess, onError) { onApproved, onFailure ->
            walletConnectClient.approveSession(
                proposal = proposal,
                namespaces = sessionNamespaces,
                properties = sessionProperties,
                onSuccess = onApproved,
                onError = onFailure,
            )
        }
    }

    override fun rejectConnection(
        proposal: WalletConnectSessionProposal,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        walletConnectClient.rejectSession(proposal, onSuccess, onError)
    }

    override fun approveAuthentication(
        request: WalletConnectAuthenticationRequest,
        auths: List<WalletConnectAuthObject>,
        wallet: Wallet,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        approveAndStoreSession(wallet, "Authentication failed", onSuccess, onError) { onApproved, onFailure ->
            walletConnectClient.approveAuthentication(
                request = request,
                auths = auths,
                onSuccess = onApproved,
                onError = onFailure,
            )
        }
    }

    override fun rejectAuthentication(
        request: WalletConnectAuthenticationRequest,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        walletConnectClient.rejectAuthentication(request, onSuccess, onError)
    }

    override fun respond(
        topic: String,
        id: Long,
        response: WalletConnectJsonRpcResponse,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        walletConnectClient.respondSessionRequest(topic, id, response, onSuccess, onError)
    }

    override fun authPayloadParams(
        payloadParams: WalletConnectAuthPayloadParams,
        supportedChains: List<String>,
        supportedMethods: List<String>,
    ): WalletConnectAuthPayloadParams {
        return walletConnectClient.generateAuthPayloadParams(payloadParams, supportedChains, supportedMethods)
    }

    override fun authMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String {
        return walletConnectClient.formatAuthMessage(payloadParams, issuer)
    }

    override fun authObject(
        payloadParams: WalletConnectAuthPayloadParams,
        issuer: String,
        signature: String,
    ): WalletConnectAuthObject {
        return walletConnectClient.generateAuthObject(payloadParams, issuer, signature)
    }

    private fun initWalletConnect(onSuccess: () -> Unit = {}, onError: (String) -> Unit = {}) {
        if (isWalletConnectInit.value) {
            onSuccess()
            return
        }
        walletConnectClient.initialize(
            onSuccess = {
                isWalletConnectInit.update { true }
                onSuccess()
            },
            onError = onError,
        )
    }

    private suspend fun sync() {
        val sessions = activeSessions() ?: return
        walletConnectService.updateSessions(sessions.mapNotNull { it.toConnectionSession(walletConnectService)?.toJson() })
    }

    private fun handlePendingRequests() {
        for (session in activeSessions().orEmpty()) {
            val request = walletConnectClient.pendingSessionRequests(session.topic).firstOrNull() ?: continue
            val verifyContext = walletConnectClient.verifyContext(request.request.id) ?: continue
            pendingEvents.tryEmit(WalletConnectEvent.SessionRequest(request, verifyContext))
        }
    }

    private fun pingActiveSessions() {
        for (session in activeSessions().orEmpty()) {
            walletConnectClient.pingSession(session.topic)
        }
    }

    private fun activeSessions(): List<WalletConnectSession>? =
        runCatching { walletConnectClient.activeSessions().filter { it.metadata != null } }
            .onFailure { Log.e("WalletConnectCoordinator", "Failed to get active sessions", it) }
            .getOrNull()

    private fun approveAndStoreSession(
        wallet: Wallet,
        failureMessage: String,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
        approve: (onSuccess: () -> Unit, onError: (String) -> Unit) -> Unit,
    ) {
        val activeBefore = activeSessions().orEmpty().map { it.topic }.toSet()
        approve(
            { persistNewSessions(wallet, activeBefore, failureMessage, onSuccess, onError) },
            onError,
        )
    }

    private fun persistNewSessions(
        wallet: Wallet,
        activeBefore: Set<String>,
        failureMessage: String,
        onSuccess: () -> Unit,
        onError: (String) -> Unit,
    ) {
        scope.launch(Dispatchers.IO) {
            runCatching {
                addNewSessions(wallet, activeBefore)
            }.onSuccess {
                onSuccess()
            }.onFailure { error ->
                onError(error.message ?: failureMessage)
            }
        }
    }

    private suspend fun addNewSessions(wallet: Wallet, activeBefore: Set<String>) {
        activeSessions().orEmpty()
            .filter { it.topic !in activeBefore }
            .mapNotNull { it.toConnectionSession(walletConnectService) }
            .forEach { session ->
                walletConnectService.addConnection(WalletConnection(session = session, wallet = wallet).toJson())
            }
    }
}
