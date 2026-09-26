package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.toJsonRpcResponse
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WalletConnectorRequestUIState
import com.gemwallet.android.features.wallet_connector.viewmodels.models.map
import com.gemwallet.android.ui.localization.text
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import javax.inject.Inject

@HiltViewModel
class WalletConnectorRequestViewModel @Inject constructor(
    private val service: GemWalletConnectServiceInterface,
    private val respondWalletConnectRequest: RespondWalletConnectRequest,
    private val pendingRequests: WalletConnectPendingRequests,
    private val activeRequest: ActiveWalletConnectRequest,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val state = MutableStateFlow(RequestViewModelState())
    private var requestJob: Job? = null

    private val request = combine(state, pendingRequests.current) { state, pending ->
        state.approved ?: pending?.takeIf { it.sessionId == state.sessionRequest?.topic }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState: StateFlow<WalletConnectorRequestUIState> = request
        .map { it.uiState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, WalletConnectorRequestUIState.Loading)

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

    fun onResult(result: String) {
        if (state.value.responseState == RequestResponseState.Responding) {
            return
        }
        val request = request.value ?: return
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

private fun WalletConnectPendingRequest?.uiState(): WalletConnectorRequestUIState = when (this) {
    null -> WalletConnectorRequestUIState.Loading
    is WalletConnectPendingRequest.SignMessage -> WalletConnectorRequestUIState.SignMessage(this)
    is WalletConnectPendingRequest.Transaction -> WalletConnectorRequestUIState.Transaction(ConfirmTransferInput(transfer, wallet), simulation)
}

private data class RequestViewModelState(val sessionRequest: WalletConnectSessionRequest? = null, val approved: WalletConnectPendingRequest? = null, val responseState: RequestResponseState = RequestResponseState.Idle)

private enum class RequestResponseState {
    Idle,
    Responding,
}
