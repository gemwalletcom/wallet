package com.gemwallet.android.features.bridge.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.services.GemSignMessageOperator
import com.gemwallet.android.data.repositories.bridge.ActiveWalletConnectRequest
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectJsonRpcResponse
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequests
import com.gemwallet.android.data.repositories.bridge.WalletConnectRequestHandler
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WCRequest
import com.gemwallet.android.features.bridge.viewmodels.model.WalletConnectOriginVerifier
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.hasCriticalWarning
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemExplorerService
import javax.inject.Inject

@HiltViewModel
class WCRequestViewModel @Inject constructor(
    private val bridgeRepository: BridgesRepository,
    private val requestHandler: WalletConnectRequestHandler,
    private val pendingRequests: WalletConnectPendingRequests,
    private val passwordStore: PasswordStore,
    private val signMessageOperator: GemSignMessageOperator,
    private val explorerService: GemExplorerService,
    private val originVerifier: WalletConnectOriginVerifier,
    private val activeRequest: ActiveWalletConnectRequest,
) : ViewModel() {

    private val state = MutableStateFlow(RequestViewModelState())
    private var requestJob: Job? = null

    val sceneState = combine(state, pendingRequests.current) { state, pending ->
        state.toSceneState(pending?.takeIf { it.sessionId == state.sessionRequest?.topic }?.let(::toRequest))
    }.stateIn(viewModelScope, SharingStarted.Eagerly, RequestSceneState.Loading)

    val buttonState = sceneState.map { scene ->
        val request = (scene as? RequestSceneState.Content)?.request as? WCRequest.SignMessage
        buttonState(
            enabled = request?.simulation?.warnings?.hasCriticalWarning() != true,
            loading = scene is RequestSceneState.Responding,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Enabled)

    fun onRequest(
        sessionRequest: WalletConnectSessionRequest,
        verifyContext: WalletConnectVerifyContext,
        onNotify: (BridgeRequestError) -> Unit,
        onError: (String) -> Unit,
    ) {
        requestJob?.cancel()
        state.update { RequestViewModelState(sessionRequest = sessionRequest) }
        Log.d(TAG, "Resolving request method=${sessionRequest.request.method} chainId=${sessionRequest.chainId} id=${sessionRequest.request.id}")
        val job = viewModelScope.launch(Dispatchers.IO) {
            val connection = bridgeRepository.getConnectionByTopic(sessionRequest.topic)
            if (connection == null) {
                rejectRequest(sessionRequest)
                return@launch
            }
            val appMetadata = connection.session.metadata
            if (originVerifier.verify(appMetadata.url, verifyContext).isScam) {
                Log.e(TAG, "Request rejected method=${sessionRequest.request.method} id=${sessionRequest.request.id}: malicious session")
                onNotify(BridgeRequestError.MaliciousSession)
                rejectRequest(sessionRequest)
                return@launch
            }
            state.update { it.copy(walletName = connection.wallet.name) }
            val response = try {
                requestHandler.handle(sessionRequest, appMetadata.url)
            } catch (err: CancellationException) {
                throw err
            }
            respond(sessionRequest, response, onError)
        }
        requestJob = job
        job.invokeOnCompletion {
            if (requestJob === job) {
                requestJob = null
            }
        }
    }

    fun onSign(onError: (String) -> Unit) {
        if (state.value.responseState == RequestResponseState.Responding) {
            return
        }
        val request = (sceneState.value as? RequestSceneState.Content)?.request as? WCRequest.SignMessage ?: return
        state.update { it.copy(responseState = RequestResponseState.Responding) }
        viewModelScope.launch(Dispatchers.IO) {
            val signature = try {
                signMessageOperator.sign(request.signer, request.wallet, passwordStore.getPassword(request.wallet.id.id))
            } catch (err: Throwable) {
                Log.e(TAG, "Sign message failed topic=${request.pending.sessionId}", err)
                state.update { it.copy(responseState = RequestResponseState.Idle) }
                onError(err.message ?: "Sign failed")
                request.reject()
                return@launch
            }
            request.approve(signature)
        }
    }

    fun onTransactionResult(result: String) {
        if (state.value.responseState == RequestResponseState.Responding) {
            return
        }
        val request = (sceneState.value as? RequestSceneState.Content)?.request as? WCRequest.Transaction ?: return
        state.update { it.copy(responseState = RequestResponseState.Responding) }
        request.approve(result)
    }

    fun onReject() {
        if (state.value.responseState == RequestResponseState.Responding) {
            return
        }
        val pending = pendingRequests.current.value
        if (pending != null && pending.sessionId == state.value.sessionRequest?.topic) {
            pending.reject()
            return
        }
        requestJob?.cancel()
        val sessionRequest = state.value.sessionRequest ?: return
        rejectRequest(sessionRequest)
    }

    fun reset() {
        requestJob?.cancel()
        requestJob = null
        state.update { RequestViewModelState() }
    }

    private fun toRequest(pending: WalletConnectPendingRequest): WCRequest = when (pending) {
        is WalletConnectPendingRequest.SignMessage -> WCRequest.SignMessage(pending, explorerService.getExplorerName(pending.chain.string))
        is WalletConnectPendingRequest.Transaction -> WCRequest.Transaction(pending)
    }

    private fun respond(sessionRequest: WalletConnectSessionRequest, response: WalletConnectJsonRpcResponse, onError: (String) -> Unit) {
        bridgeRepository.respondSessionRequest(
            topic = sessionRequest.topic,
            id = sessionRequest.request.id,
            response = response,
            onSuccess = { activeRequest.finish(sessionRequest) },
            onError = { error ->
                state.update { it.copy(responseState = RequestResponseState.Idle) }
                onError(error.ifBlank { "Request failed" })
            },
        )
    }

    private fun rejectRequest(sessionRequest: WalletConnectSessionRequest) {
        bridgeRepository.respondSessionRequest(
            topic = sessionRequest.topic,
            id = sessionRequest.request.id,
            response = WalletConnectJsonRpcResponse.Error(
                code = 4001,
                message = "User rejected the request",
            ),
            onSuccess = { activeRequest.finish(sessionRequest) },
            onError = { error -> Log.e(TAG, "Request rejection failed id=${sessionRequest.request.id}: $error") },
        )
    }

    private companion object {
        const val TAG = "WalletConnect"
    }
}

private data class RequestViewModelState(
    val sessionRequest: WalletConnectSessionRequest? = null,
    val walletName: String? = null,
    val responseState: RequestResponseState = RequestResponseState.Idle,
) {
    fun toSceneState(request: WCRequest?): RequestSceneState {
        request ?: return RequestSceneState.Loading
        val walletName = walletName ?: return RequestSceneState.Loading
        val requestState = RequestSceneState.Request(walletName = walletName, request = request)
        return when (responseState) {
            RequestResponseState.Idle -> requestState
            RequestResponseState.Responding -> RequestSceneState.Responding(requestState)
        }
    }
}

private enum class RequestResponseState {
    Idle,
    Responding,
}

sealed interface RequestSceneState {
    data object Loading : RequestSceneState

    sealed interface Content : RequestSceneState {
        val walletName: String
        val request: WCRequest
    }

    class Request(
        override val walletName: String,
        override val request: WCRequest,
    ) : Content

    class Responding(
        private val requestState: Request,
    ) : Content {
        override val walletName: String get() = requestState.walletName
        override val request: WCRequest get() = requestState.request
    }
}
