package com.gemwallet.android.features.bridge.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.getKeystorePassword
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.toJsonRpcResponse
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.features.bridge.viewmodels.model.WCRequest
import com.gemwallet.android.features.bridge.viewmodels.model.map
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
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WCRequestViewModel @Inject constructor(
    private val service: GemWalletConnectServiceInterface,
    private val respondWalletConnectRequest: RespondWalletConnectRequest,
    private val pendingRequests: WalletConnectPendingRequests,
    private val signMessageService: GemSignMessageServiceInterface,
    private val activeRequest: ActiveWalletConnectRequest,
) : ViewModel() {

    private val state = MutableStateFlow(RequestViewModelState())
    private var requestJob: Job? = null

    private val request = combine(state, pendingRequests.current) { state, pending ->
        state.approved ?: pending?.takeIf { it.sessionId == state.sessionRequest?.topic }?.let(::toRequest)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val payloadAddressNames = request
        .map { it as? WCRequest.SignMessage }
        .distinctUntilChanged { old, new -> old?.pending === new?.pending }
        .mapLatest { request -> request?.addressNames().orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    val sceneState = combine(state, request, payloadAddressNames) { state, request, addressNames ->
        state.toSceneState((request as? WCRequest.SignMessage)?.withAddressNames(addressNames) ?: request)
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
        if (requestJob != null && state.value.sessionRequest == sessionRequest) {
            return
        }
        requestJob?.cancel()
        pendingRequests.current.value?.takeIf { it.sessionId == sessionRequest.topic }?.reject()
        state.update { RequestViewModelState(sessionRequest = sessionRequest) }
        Log.d(TAG, "Resolving request method=${sessionRequest.request.method} chainId=${sessionRequest.chainId} id=${sessionRequest.request.id}")
        val job = viewModelScope.launch(Dispatchers.IO) {
            val outcome = service.processRequest(
                GemWalletConnectSessionRequest(
                    topic = sessionRequest.topic,
                    requestId = sessionRequest.request.id.toString(),
                    method = sessionRequest.request.method,
                    params = sessionRequest.request.params,
                    chainId = sessionRequest.chainId,
                    origin = verifyContext.origin,
                    validation = verifyContext.map(),
                ),
            )
            when (val failure = outcome.failure) {
                null -> Unit
                GemWalletConnectFailure.MaliciousOrigin -> onNotify(BridgeRequestError.MaliciousSession)
                is GemWalletConnectFailure.Failed -> onError(failure.message)
            }
            when (val response = outcome.response) {
                null -> activeRequest.finish(sessionRequest)
                else -> respond(sessionRequest, response.toJsonRpcResponse(), onError)
            }
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
        state.update { it.copy(responseState = RequestResponseState.Responding, approved = request) }
        viewModelScope.launch(Dispatchers.IO) {
            val signature = try {
                signMessageService.sign(request.wallet.id.id, request.signMessage)
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                Log.e(TAG, "Sign message failed topic=${request.pending.sessionId}", err)
                state.update { it.copy(responseState = RequestResponseState.Idle, approved = null) }
                onError(err.message.orEmpty())
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
        state.update { it.copy(responseState = RequestResponseState.Responding, approved = request) }
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
        respond(sessionRequest, service.userRejectedError().toJsonRpcResponse(), onError = { Log.e(TAG, "Request rejection failed id=${sessionRequest.request.id}: $it") })
    }

    private fun toRequest(pending: WalletConnectPendingRequest): WCRequest = when (pending) {
        is WalletConnectPendingRequest.SignMessage -> WCRequest.SignMessage(pending, signMessageService)
        is WalletConnectPendingRequest.Transaction -> WCRequest.Transaction(pending)
    }

    private fun respond(sessionRequest: WalletConnectSessionRequest, response: WalletConnectJsonRpcResponse, onError: (String) -> Unit) {
        respondWalletConnectRequest.respond(
            topic = sessionRequest.topic,
            id = sessionRequest.request.id,
            response = response,
            onSuccess = { activeRequest.finish(sessionRequest) },
            onError = { error ->
                activeRequest.finish(sessionRequest)
                state.update { it.copy(responseState = RequestResponseState.Idle, approved = null) }
                onError(error)
            },
        )
    }

    private companion object {
        const val TAG = "WalletConnect"
    }
}

private data class RequestViewModelState(
    val sessionRequest: WalletConnectSessionRequest? = null,
    val approved: WCRequest? = null,
    val responseState: RequestResponseState = RequestResponseState.Idle,
) {
    fun toSceneState(request: WCRequest?): RequestSceneState {
        request ?: return RequestSceneState.Loading
        val requestState = RequestSceneState.Request(walletName = request.wallet.name, request = request)
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
