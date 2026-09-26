package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.getKeystorePassword
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.toJsonRpcResponse
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.wallet_connector.viewmodels.models.ReviewTexts
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WCRequest
import com.gemwallet.android.features.wallet_connector.viewmodels.models.map
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemSignerFailure
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import uniffi.gemstone.applicationConnectionRow
import uniffi.gemstone.signerFailure
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WCRequestViewModel @Inject constructor(
    private val service: GemWalletConnectServiceInterface,
    private val signMessageService: GemSignMessageServiceInterface,
    private val respondWalletConnectRequest: RespondWalletConnectRequest,
    private val pendingRequests: WalletConnectPendingRequests,
    private val activeRequest: ActiveWalletConnectRequest,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val state = MutableStateFlow(RequestViewModelState())
    private var requestJob: Job? = null

    private val request = combine(state, pendingRequests.current) { state, pending ->
        state.approved ?: pending?.takeIf { it.sessionId == state.sessionRequest?.topic }?.let(::toRequest)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val namedRequest = request
        .map { it as? WCRequest.SignMessage }
        .distinctUntilChanged { old, new -> old?.pending === new?.pending }
        .mapLatest { request -> request?.withAddressNames() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sceneState = combine(state, request, namedRequest) { state, request, named ->
        state.toSceneState(named?.takeIf { it.pending === request?.pending } ?: request)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, RequestSceneState.Loading)

    val buttonState = sceneState.map { scene ->
        val request = (scene as? RequestSceneState.Content)?.request as? WCRequest.SignMessage
        buttonState(
            enabled = request?.hasCriticalWarning != true,
            loading = scene is RequestSceneState.Responding,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Enabled)

    fun onRequest(sessionRequest: WalletConnectSessionRequest, verifyContext: WalletConnectVerifyContext, onNotify: (String) -> Unit, onError: (String) -> Unit) {
        if (state.value.sessionRequest == sessionRequest) {
            return
        }
        requestJob?.cancel()
        pendingRequests.current.value?.takeIf { it.sessionId == sessionRequest.topic }?.reject()
        state.update { RequestViewModelState(sessionRequest = sessionRequest) }
        val job = viewModelScope.launch {
            val outcome = withContext(ioDispatcher) {
                service.requestOutcome(
                    GemWalletConnectSessionRequest(
                        topic = sessionRequest.topic,
                        requestId = sessionRequest.request.id.toString(),
                        method = sessionRequest.request.method,
                        params = sessionRequest.request.params,
                        chainId = sessionRequest.chainId,
                        origin = verifyContext.origin,
                        validation = verifyContext.map(),
                        expiry = null,
                    ),
                )
            }
            when (val failure = outcome.failure) {
                null -> Unit
                GemWalletConnectFailure.MaliciousOrigin, GemWalletConnectFailure.Expired -> onNotify(failure.text(context))
                is GemWalletConnectFailure.Failed -> onError(failure.text(context))
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
        viewModelScope.launch(ioDispatcher) {
            val signature = try {
                service.signMessage(request.wallet.id.id, request.signMessage)
            } catch (err: GemServiceException) {
                Log.e(TAG, "Sign message failed topic=${request.pending.sessionId}", err)
                state.update { it.copy(responseState = RequestResponseState.Idle, approved = null) }
                when (val failure = signerFailure(err.text())) {
                    is GemSignerFailure.Retry -> onError(failure.error.text(context))
                    GemSignerFailure.Reject -> request.reject()
                }
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
        respond(sessionRequest, WalletConnectJsonRpcResponse.Error(code = GemConstants.walletConnectUserRejectedErrorCode, message = GemConstants.walletConnectUserRejectedErrorMessage), onError = {
            Log.e(TAG, "Request rejection failed id=${sessionRequest.request.id}: $it")
        })
    }

    private fun toRequest(pending: WalletConnectPendingRequest): WCRequest {
        val row = applicationConnectionRow(pending.appMetadata.toGem())
        return when (pending) {
            is WalletConnectPendingRequest.SignMessage -> WCRequest.SignMessage(pending, row, signMessageService, ReviewTexts(context), context)
            is WalletConnectPendingRequest.Transaction -> WCRequest.Transaction(pending, row)
        }
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
                onError(error.text(context))
            },
        )
    }

    private companion object {
        const val TAG = "WalletConnect"
    }
}

private data class RequestViewModelState(val sessionRequest: WalletConnectSessionRequest? = null, val approved: WCRequest? = null, val responseState: RequestResponseState = RequestResponseState.Idle) {
    fun toSceneState(request: WCRequest?): RequestSceneState {
        request ?: return RequestSceneState.Loading
        val requestState = RequestSceneState.Request(request = request)
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
        val request: WCRequest
    }

    class Request(override val request: WCRequest) : Content

    class Responding(private val requestState: Request) : Content {
        override val request: WCRequest get() = requestState.request
    }
}
