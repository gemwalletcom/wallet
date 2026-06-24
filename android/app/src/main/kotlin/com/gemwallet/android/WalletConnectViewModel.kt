package com.gemwallet.android

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectEvent
import com.gemwallet.android.data.repositories.bridge.WalletConnectAuthenticationRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectJsonRpcResponse
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionProposal
import com.gemwallet.android.data.repositories.bridge.WalletConnectSessionRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectVerifyContext
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@HiltViewModel
class WalletConnectViewModel @Inject constructor(
    private val bridgesRepository: BridgesRepository,
    activeRequestState: WalletConnectActiveRequestState,
) : ViewModel() {

    private val _uiState = MutableStateFlow<WalletConnectIntent>(WalletConnectIntent.Idle)
    val uiState = _uiState.asStateFlow()

    init {
        bridgesRepository.bridgeEvents
            .onEach { event -> event.toUIState()?.let { intent -> _uiState.update { intent } } }
            .launchIn(viewModelScope)

        _uiState
            .onEach { intent -> activeRequestState.setActive(intent.requiresUserAction) }
            .launchIn(viewModelScope)
    }

    fun onCancel() {
        _uiState.update { WalletConnectIntent.Idle }
    }

    fun rejectSessionRequest(request: WalletConnectSessionRequest) {
        bridgesRepository.respondSessionRequest(
            topic = request.topic,
            id = request.request.id,
            response = WalletConnectJsonRpcResponse.Error(
                code = 4001,
                message = "User rejected the request",
            ),
            onSuccess = {},
            onError = {},
        )
        onCancel()
    }

    fun rejectSessionProposal(proposal: WalletConnectSessionProposal) {
        bridgesRepository.rejectConnection(proposal, onSuccess = {}, onError = {})
        onCancel()
    }

    fun rejectSessionAuthenticate(request: WalletConnectAuthenticationRequest) {
        bridgesRepository.rejectAuthentication(request, onSuccess = {}, onError = {})
        onCancel()
    }
}

sealed interface WalletConnectIntent {

    val requiresUserAction: Boolean

    data object Idle : WalletConnectIntent {
        override val requiresUserAction = false
    }

    data object Cancel : WalletConnectIntent {
        override val requiresUserAction = false
    }

    class SessionRequest(val request: WalletConnectSessionRequest, val verifyContext: WalletConnectVerifyContext?) : WalletConnectIntent {
        override val requiresUserAction = true
    }

    class AuthRequest(val request: WalletConnectAuthenticationRequest, val verifyContext: WalletConnectVerifyContext?) : WalletConnectIntent {
        override val requiresUserAction = true
    }

    class SessionProposal(val sessionProposal: WalletConnectSessionProposal, val verifyContext: WalletConnectVerifyContext?) : WalletConnectIntent {
        override val requiresUserAction = true
    }
}

private fun WalletConnectEvent.toUIState(): WalletConnectIntent? {
    return when (this) {
        is WalletConnectEvent.SessionRequest -> WalletConnectIntent.SessionRequest(request, verifyContext)
        is WalletConnectEvent.AuthenticationRequest -> WalletConnectIntent.AuthRequest(request, verifyContext)
        is WalletConnectEvent.SessionProposal -> WalletConnectIntent.SessionProposal(proposal, verifyContext)
        else -> null
    }
}
