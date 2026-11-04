package com.gemwallet.android.features.wallet_connector.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WalletConnectorRequestUIState
import com.gemwallet.android.ui.localization.text
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@HiltViewModel
class WalletConnectorRequestViewModel @Inject constructor(
    private val respondWalletConnectRequest: RespondWalletConnectRequest,
    private val pendingRequests: WalletConnectPendingRequests,
    private val activeRequest: ActiveWalletConnectRequest,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val state = MutableStateFlow(RequestViewModelState())

    private val request = combine(state, pendingRequests.current) { state, pending ->
        state.approved ?: pending?.takeIf { it.sessionId == state.sessionRequest?.topic }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState: StateFlow<WalletConnectorRequestUIState> = request
        .map { it.uiState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, WalletConnectorRequestUIState.Loading)

    fun onRequest(sessionRequest: WalletConnectSessionRequest, verifyContext: WalletConnectVerifyContext, onError: (String) -> Unit) {
        if (state.value.sessionRequest == sessionRequest) {
            return
        }
        pendingRequests.current.value?.takeIf { it.sessionId == sessionRequest.topic }?.reject()
        state.update { RequestViewModelState(sessionRequest = sessionRequest) }
        respondWalletConnectRequest.respondRequest(sessionRequest, verifyContext, onFailure = { onError(it.text(context)) }) {
            state.update { it.copy(responseState = RequestResponseState.Finished) }
            activeRequest.finish(sessionRequest)
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
        val current = state.value
        val sessionRequest = current.sessionRequest ?: return
        if (current.responseState != RequestResponseState.Idle) {
            return
        }
        val pending = pendingRequests.current.value
        if (pending != null && pending.sessionId == sessionRequest.topic) {
            pending.reject()
            return
        }
        respondWalletConnectRequest.rejectRequest(sessionRequest) { activeRequest.finish(sessionRequest) }
    }

    override fun onCleared() {
        onReject()
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
    Finished,
}
