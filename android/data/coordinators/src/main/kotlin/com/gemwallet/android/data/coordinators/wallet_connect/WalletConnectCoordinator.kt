package com.gemwallet.android.data.coordinators.wallet_connect

import android.util.Log
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnectAuthentication
import com.gemwallet.android.application.wallet_connect.cases.ApproveWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.DisconnectWalletConnection
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.SyncWalletConnectSessions
import com.gemwallet.android.data.services.gemstone.stream.WebSocketEvent
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectAuthMessage
import uniffi.gemstone.GemWalletConnectAuthentication
import uniffi.gemstone.GemWalletConnectClientInterface
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectProposal
import uniffi.gemstone.GemWalletConnectRejectionReason
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import java.util.concurrent.ConcurrentHashMap

class WalletConnectCoordinator(
    private val client: GemWalletConnectClientInterface,
    private val relay: WalletConnectRelay,
    private val walletConnectService: GemWalletConnectServiceInterface,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) : PairWalletConnect,
    SyncWalletConnectSessions,
    DisconnectWalletConnection,
    ApproveWalletConnection,
    ApproveWalletConnectAuthentication,
    RespondWalletConnectRequest {

    private val requestJobs = ConcurrentHashMap<GemWalletConnectSessionRequest, Job>()
    private var connectionJob: Job? = null

    init {
        scope.launch {
            runCatchingCancellable {
                if (walletConnectService.hasSessions()) {
                    client.restore()
                }
            }.onFailure { Log.e(TAG, "Restore sessions failed", it) }
        }
    }

    fun start() {
        scope.launch {
            runCatchingCancellable {
                if (client.needsConnection()) {
                    connect()
                }
            }.onFailure { Log.e(TAG, "Start relay connection failed", it) }
        }
    }

    @Synchronized
    fun stop() {
        connectionJob?.cancel()
        connectionJob = null
    }

    override suspend fun syncSessions() {
        runCatchingCancellable { client.restore() }
            .onFailure { Log.e(TAG, "Restore sessions failed", it) }
    }

    override suspend fun disconnect(connectionId: String, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) {
        runCatchingCancellable {
            connect()
            client.disconnect(connectionId)
        }.onSuccess { onSuccess() }
            .onFailure { onError(it.errorText()) }
    }

    override fun pair(uri: String, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) = launchCall(onSuccess, onError) {
        connect()
        client.pair(uri)
    }

    override fun approveConnection(wallet: Wallet, proposal: GemWalletConnectProposal, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) = launchCall(onSuccess, onError) {
        client.approveSession(proposal.id, wallet.toGem())
    }

    override fun rejectConnection(proposal: GemWalletConnectProposal, reason: GemWalletConnectRejectionReason, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) = launchCall(onSuccess, onError) {
        client.rejectSession(proposal.id, reason)
    }

    override fun authMessage(authentication: GemWalletConnectAuthentication, wallet: Wallet): GemWalletConnectAuthMessage = client.authenticationMessage(authentication.id, wallet.toGem())

    override fun approveAuthentication(authentication: GemWalletConnectAuthentication, wallet: Wallet, signature: String, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) = launchCall(onSuccess, onError) {
        client.approveAuthentication(authentication.id, wallet.toGem(), signature)
    }

    override fun rejectAuthentication(authentication: GemWalletConnectAuthentication, onSuccess: () -> Unit, onError: (GemErrorText) -> Unit) = launchCall(onSuccess, onError) {
        client.rejectAuthentication(authentication.id)
    }

    override fun respondRequest(request: GemWalletConnectSessionRequest, onFailure: (GemWalletConnectFailure) -> Unit, onFinish: () -> Unit) {
        val job = scope.launch {
            runCatchingCancellable { client.answerRequest(request.topic, request.requestId) }
                .onSuccess { failure -> failure?.let(onFailure) }
                .onFailure { Log.e(TAG, "Answer request ${request.requestId} failed", it) }
            onFinish()
        }
        requestJobs[request] = job
        job.invokeOnCompletion { requestJobs.remove(request, job) }
    }

    override fun rejectRequest(request: GemWalletConnectSessionRequest, onFinish: () -> Unit) {
        val job = requestJobs.remove(request) ?: return
        job.cancel()
        scope.launch {
            runCatchingCancellable { client.rejectRequest(request.topic, request.requestId) }
                .onFailure { Log.e(TAG, "Reject request ${request.requestId} failed", it) }
            onFinish()
        }
    }

    private fun launchCall(onSuccess: () -> Unit, onError: (GemErrorText) -> Unit, call: suspend () -> Unit) {
        scope.launch {
            runCatchingCancellable { call() }
                .onSuccess { onSuccess() }
                .onFailure { onError(it.errorText()) }
        }
    }

    @Synchronized
    private fun connect() {
        if (connectionJob?.isActive == true) {
            return
        }
        connectionJob = scope.launch {
            relay.connect().collect { event ->
                runCatchingCancellable {
                    when (event) {
                        WebSocketEvent.Connected -> client.connected()
                        is WebSocketEvent.Message -> client.receive(event.text)
                        WebSocketEvent.Disconnected -> Unit
                    }
                }.onFailure { Log.e(TAG, "Relay event failed", it) }
            }
        }
    }

    private companion object {
        const val TAG = "WalletConnect"
    }
}
