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
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.SyncWalletConnectSessions
import com.gemwallet.android.application.wallet_connect.toConnectionSession
import com.gemwallet.android.application.wallet_connect.toSupportedNamespaces
import com.gemwallet.android.data.services.gemstone.stores.GemstoneConnectionStore
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnection
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.merge
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemChainServiceInterface
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.walletConnectErrorText

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectCoordinator(
    private val connectionStore: GemstoneConnectionStore,
    private val walletConnectClient: WalletConnectClient,
    private val walletConnectService: GemWalletConnectServiceInterface,
    private val chainService: GemChainServiceInterface,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO + SupervisorJob()),
) : IsWalletConnectEnabled,
    PairWalletConnect,
    SyncWalletConnectSessions,
    DisconnectWalletConnection,
    ApproveWalletConnection,
    ApproveWalletConnectAuthentication,
    RespondWalletConnectRequest {

    private val pendingEvents = MutableSharedFlow<WalletConnectEvent>(extraBufferCapacity = 16)
    private val isWalletConnectInit = MutableStateFlow(false)
    private val approvingWallet = MutableStateFlow<Wallet?>(null)
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
                emitPendingRequests()
            }
        }
        scope.launch(Dispatchers.IO) {
            bridgeEvents.collect { event ->
                when (event) {
                    is WalletConnectEvent.SessionDeleted -> walletConnectService.deleteSession(event.topic)
                    is WalletConnectEvent.SessionSettled -> storeSettledSession(event.session)
                    is WalletConnectEvent.SessionChanged -> sync()
                    else -> Unit
                }
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
                } catch (err: Throwable) {
                    onError(err.errorText())
                }
            },
            onError = { onError(clientErrorText(it)) },
        )
    }

    override fun approveConnection(wallet: Wallet, proposal: WalletConnectSessionProposal, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
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
        approveAndStoreSession(wallet, onSuccess, onError) { onApproved, onFailure ->
            walletConnectClient.approveSession(
                proposal = proposal,
                namespaces = sessionNamespaces,
                properties = sessionProperties,
                onSuccess = onApproved,
                onError = onFailure,
            )
        }
    }

    override fun rejectConnection(proposal: WalletConnectSessionProposal, reason: GemWalletConnectRejectionReason, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        val rejection = walletConnectService.sessionRejection(reason)
        walletConnectClient.rejectSession(
            proposal = proposal,
            rejection = rejection,
            onSuccess = {
                if (rejection.deletesSession) {
                    scope.launch {
                        runCatching { walletConnectService.deleteSession(proposal.pairingTopic) }
                            .onFailure { Log.e("WalletConnect", "Delete rejected session failed", it) }
                    }
                }
                onSuccess()
            },
            onError = { onError(clientErrorText(it)) },
        )
    }

    override fun approveAuthentication(request: WalletConnectAuthenticationRequest, auths: List<WalletConnectAuthObject>, wallet: Wallet, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        approveAndStoreSession(wallet, onSuccess, onError) { onApproved, onFailure ->
            walletConnectClient.approveAuthentication(
                request = request,
                auths = auths,
                onSuccess = onApproved,
                onError = onFailure,
            )
        }
    }

    override fun rejectAuthentication(request: WalletConnectAuthenticationRequest, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        walletConnectClient.rejectAuthentication(request, onSuccess) { onError(clientErrorText(it)) }
    }

    override fun respond(topic: String, id: Long, response: WalletConnectJsonRpcResponse, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        walletConnectClient.respondSessionRequest(topic, id, response, onSuccess) { onError(clientErrorText(it)) }
    }

    override fun authPayloadParams(payloadParams: WalletConnectAuthPayloadParams, supportedChains: List<String>, supportedMethods: List<String>): WalletConnectAuthPayloadParams =
        walletConnectClient.generateAuthPayloadParams(payloadParams, supportedChains, supportedMethods)

    override fun authMessage(payloadParams: WalletConnectAuthPayloadParams, issuer: String): String = walletConnectClient.formatAuthMessage(payloadParams, issuer)

    override fun authObject(payloadParams: WalletConnectAuthPayloadParams, issuer: String, signature: String): WalletConnectAuthObject = walletConnectClient.generateAuthObject(payloadParams, issuer, signature)

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

    private fun approveAndStoreSession(wallet: Wallet, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit, approve: (onSuccess: () -> Unit, onError: (String) -> Unit) -> Unit) {
        approvingWallet.value = wallet
        val activeBefore = activeSessions().orEmpty().map { it.topic }.toSet()
        approve(
            { persistNewSessions(wallet, activeBefore, onSuccess, onError) },
            { onError(clientErrorText(it)) },
        )
    }

    private fun persistNewSessions(wallet: Wallet, activeBefore: Set<String>, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        scope.launch(Dispatchers.IO) {
            runCatching {
                addNewSessions(wallet, activeBefore)
            }.onSuccess {
                onSuccess()
            }.onFailure { error ->
                onError(error.errorText())
            }
        }
    }

    private suspend fun addNewSessions(wallet: Wallet, activeBefore: Set<String>) {
        activeSessions().orEmpty()
            .filter { it.topic !in activeBefore }
            .forEach { storeSession(it, wallet) }
    }

    private suspend fun storeSettledSession(session: WalletConnectSession) {
        val wallet = approvingWallet.value ?: return
        storeSession(session, wallet)
    }

    private suspend fun storeSession(session: WalletConnectSession, wallet: Wallet) {
        if (connectionStore.getConnectionBySessionId(session.topic) != null) {
            return
        }
        val connectionSession = session.toConnectionSession(walletConnectService) ?: return
        walletConnectService.addConnection(WalletConnection(session = connectionSession, wallet = wallet).toGem())
    }
}

private fun clientErrorText(message: String): GemErrorText = walletConnectErrorText(message)
