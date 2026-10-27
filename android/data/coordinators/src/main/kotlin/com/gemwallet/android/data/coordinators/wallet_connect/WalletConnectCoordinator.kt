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
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionNamespace
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.addingAccount
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.SyncWalletConnectSessions
import com.gemwallet.android.application.wallet_connect.map
import com.gemwallet.android.application.wallet_connect.toConnectionSession
import com.gemwallet.android.application.wallet_connect.toJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.toSupportedNamespaces
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemChainServiceInterface
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectOutcome
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.GemWalletConnectResponse
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import uniffi.gemstone.walletConnectErrorText
import java.util.concurrent.ConcurrentHashMap

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectCoordinator(
    private val walletConnectClient: WalletConnectClient,
    private val walletConnectService: GemWalletConnectServiceInterface,
    private val chainService: GemChainServiceInterface,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : IsWalletConnectEnabled,
    PairWalletConnect,
    SyncWalletConnectSessions,
    DisconnectWalletConnection,
    ApproveWalletConnection,
    ApproveWalletConnectAuthentication,
    RespondWalletConnectRequest {

    private val pendingEvents = MutableSharedFlow<WalletConnectEvent>(extraBufferCapacity = 16)
    private val isWalletConnectInit = MutableStateFlow(false)
    private val approvedWallets = ConcurrentHashMap<Set<String>, Wallet>()
    private val requestJobs = ConcurrentHashMap<WalletConnectSessionRequest, Job>()
    private val events = isWalletConnectInit.flatMapLatest {
        if (it) {
            merge(walletConnectClient.events, pendingEvents)
        } else {
            emptyFlow()
        }
    }
    val bridgeEvents = events.filter(::isFirstDelivery).flowOn(Dispatchers.IO)

    init {
        scope.launch(Dispatchers.IO) {
            runCatchingCancellable {
                if (walletConnectService.hasSessions()) {
                    initWalletConnect()
                    sync()
                    pingActiveSessions()
                    emitPendingRequests()
                }
            }.onFailure { Log.e("WalletConnect", "Restore sessions failed", it) }
        }
        scope.launch(Dispatchers.IO) {
            events.collect { event ->
                runCatchingCancellable {
                    when (event) {
                        is WalletConnectEvent.SessionDeleted -> walletConnectService.deleteSession(event.topic)
                        is WalletConnectEvent.SessionSettled -> approvedWallets[event.session.namespaces.accounts()]?.let { storeSession(event.session, it) }
                        is WalletConnectEvent.SessionChanged -> sync()
                        else -> Unit
                    }
                }.onFailure { Log.e("WalletConnect", "Handle session event failed", it) }
            }
        }
    }

    override fun isWalletConnectEnabled(): Boolean = walletConnectClient.isEnabled

    override suspend fun syncSessions() = sync()

    override suspend fun disconnect(connectionId: String, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        walletConnectService.deleteSession(connectionId)
        val activeSession = activeSessions()?.firstOrNull { it.topic == connectionId }
        if (activeSession == null) {
            onSuccess()
            return
        }
        walletConnectClient.disconnectSession(activeSession.topic, onSuccess = onSuccess, onError = { onError(clientErrorText(it)) })
    }

    override fun pair(uri: String, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        initWalletConnect(
            onSuccess = {
                try {
                    walletConnectClient.pair(
                        uri = uri,
                        onSuccess = { onSuccess() },
                        onError = { onError(clientErrorText(it)) },
                    )
                } catch (error: Throwable) {
                    onError(error.errorText())
                }
            },
            onError = { onError(clientErrorText(it)) },
        )
    }

    override fun approveConnection(wallet: Wallet, proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        scope.launch {
            runCatchingCancellable { approve(wallet, proposal, onSuccess, onError) }
                .onFailure { onError(it.errorText()) }
        }
    }

    private suspend fun approve(wallet: Wallet, proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        val approval = walletConnectService.sessionApproval(wallet = wallet.toGem())
        val sessionNamespaces = walletConnectClient.generateApprovedNamespaces(
            proposal = proposal,
            supportedNamespaces = approval.toSupportedNamespaces(chainService),
        )
        val sessionProperties = walletConnectService.configSessionProperties(
            properties = proposal.properties ?: emptyMap(),
            caip2Chains = sessionNamespaces.values.flatMap { it.chains.orEmpty() },
            accounts = approval.accounts,
        )
        approvedWallets[sessionNamespaces.accounts()] = wallet
        walletConnectClient.approveSession(
            proposal = proposal,
            namespaces = sessionNamespaces,
            properties = sessionProperties,
            onSuccess = onSuccess,
            onError = { onError(clientErrorText(it)) },
        )
    }

    override fun rejectConnection(proposal: WalletConnectSessionProposal, reason: GemWalletConnectRejectionReason, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        scope.launch {
            val rejection = walletConnectService.sessionRejection(reason)
            walletConnectClient.rejectSession(
                proposal = proposal,
                rejection = rejection,
                onSuccess = {
                    if (rejection.deletesSession) {
                        scope.launch {
                            runCatchingCancellable { walletConnectService.deleteSession(proposal.pairingTopic) }
                                .onFailure { Log.e("WalletConnect", "Delete rejected session failed", it) }
                        }
                    }
                    onSuccess()
                },
                onError = { onError(clientErrorText(it)) },
            )
        }
    }

    override fun approveAuthentication(request: WalletConnectAuthenticationRequest, auths: List<WalletConnectAuthObject>, wallet: Wallet, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        val activeBefore = activeSessions().orEmpty().map { it.topic }.toSet()
        walletConnectClient.approveAuthentication(
            request = request,
            auths = auths,
            onSuccess = { storeAuthenticatedSessions(wallet, activeBefore, onSuccess, onError) },
            onError = { onError(clientErrorText(it)) },
        )
    }

    override fun rejectAuthentication(request: WalletConnectAuthenticationRequest, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        walletConnectClient.rejectAuthentication(request, onSuccess) { onError(clientErrorText(it)) }
    }

    override fun respondRequest(request: WalletConnectSessionRequest, verifyContext: WalletConnectVerifyContext, onFailure: (GemWalletConnectFailure) -> Unit, onFinish: () -> Unit) {
        val job = scope.launch { answer(request, verifyContext, onFailure, onFinish) }
        requestJobs[request] = job
        job.invokeOnCompletion { requestJobs.remove(request, job) }
    }

    override fun rejectRequest(request: WalletConnectSessionRequest, onFinish: () -> Unit) {
        val job = requestJobs.remove(request) ?: return
        job.cancel()
        reply(request, WalletConnectJsonRpcResponse.Error(code = GemConstants.walletConnectUserRejectedErrorCode, message = GemConstants.walletConnectUserRejectedErrorMessage), onFinish)
    }

    private suspend fun answer(request: WalletConnectSessionRequest, verifyContext: WalletConnectVerifyContext, onFailure: (GemWalletConnectFailure) -> Unit, onFinish: () -> Unit) {
        when (val outcome = walletConnectService.requestOutcome(request.toGem(verifyContext))) {
            GemWalletConnectOutcome.Ignored -> onFinish()

            is GemWalletConnectOutcome.Respond -> reply(request, outcome.response.toJsonRpcResponse(), onFinish)

            is GemWalletConnectOutcome.Fail -> {
                onFailure(outcome.failure)
                reply(request, outcome.error.toJsonRpcResponse(), onFinish)
            }

            is GemWalletConnectOutcome.AddSessionAccount -> addSessionAccount(request.topic, outcome.account) {
                reply(request, GemWalletConnectResponse.Null.toJsonRpcResponse(), onFinish)
            }
        }
    }

    private fun reply(request: WalletConnectSessionRequest, response: WalletConnectJsonRpcResponse, onFinish: () -> Unit) {
        walletConnectClient.respondSessionRequest(request.topic, request.request.id, response, onFinish) { error ->
            Log.e("WalletConnect", "Respond to request ${request.request.id} failed: $error")
            onFinish()
        }
    }

    private fun addSessionAccount(topic: String, account: String, onComplete: () -> Unit) {
        val session = activeSessions()?.firstOrNull { it.topic == topic }
        if (session == null) {
            onComplete()
            return
        }
        walletConnectClient.updateSession(
            topic = topic,
            namespaces = session.namespaces.addingAccount(account),
            onSuccess = onComplete,
            onError = {
                Log.e("WalletConnect", "Add session account failed: $it")
                onComplete()
            },
        )
    }

    override fun authPayloadParams(payloadParams: WalletConnectAuthPayloadParams, supportedChains: List<String>, supportedMethods: List<String>): WalletConnectAuthPayloadParams =
        walletConnectClient.generateAuthPayloadParams(payloadParams, supportedChains, supportedMethods)

    override fun authMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String = walletConnectClient.formatAuthMessage(payloadParams, issuer)

    override fun authObject(payloadParams: WalletConnectAuthPayloadParams, issuer: String, signature: String): WalletConnectAuthObject = walletConnectClient.generateAuthObject(payloadParams, issuer, signature)

    private fun isFirstDelivery(event: WalletConnectEvent): Boolean = when (event) {
        is WalletConnectEvent.SessionProposal -> walletConnectService.shouldProcessProposal(event.proposal.proposerPublicKey)
        else -> true
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
        walletConnectService.updateSessions(sessions.mapNotNull { it.toConnectionSession(walletConnectService)?.toGem() })
    }

    private fun emitPendingRequests() {
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

    private fun activeSessions(): List<WalletConnectSession>? = runCatching { walletConnectClient.activeSessions().filter { it.metadata != null } }
        .onFailure { Log.e("WalletConnectCoordinator", "Failed to get active sessions", it) }
        .getOrNull()

    private fun storeAuthenticatedSessions(wallet: Wallet, activeBefore: Set<String>, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        scope.launch(Dispatchers.IO) {
            runCatchingCancellable {
                activeSessions().orEmpty()
                    .filter { it.topic !in activeBefore }
                    .forEach { storeSession(it, wallet) }
            }.onSuccess {
                onSuccess()
            }.onFailure { error ->
                onError(error.errorText())
            }
        }
    }

    private suspend fun storeSession(session: WalletConnectSession, wallet: Wallet) {
        val connectionSession = session.toConnectionSession(walletConnectService) ?: return
        walletConnectService.addConnection(WalletConnection(session = connectionSession, wallet = wallet).toGem())
    }
}

private fun clientErrorText(message: String): GemErrorText = walletConnectErrorText(message)

private fun WalletConnectSessionRequest.toGem(verifyContext: WalletConnectVerifyContext) = GemWalletConnectSessionRequest(
    topic = topic,
    requestId = request.id.toString(),
    method = request.method,
    params = request.params,
    chainId = chainId,
    origin = verifyContext.origin,
    validation = verifyContext.map(),
    expiry = null,
)

private fun Map<String, WalletConnectSessionNamespace>.accounts(): Set<String> = values.flatMapTo(mutableSetOf()) { it.accounts }
