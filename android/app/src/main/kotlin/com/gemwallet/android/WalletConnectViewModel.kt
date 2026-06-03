package com.gemwallet.android

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectEvent
import com.reown.walletkit.client.Wallet
import com.reown.walletkit.client.WalletKit
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

    fun rejectSessionRequest(request: Wallet.Model.SessionRequest) {
        WalletKit.respondSessionRequest(
            params = Wallet.Params.SessionRequestResponse(
                sessionTopic = request.topic,
                jsonRpcResponse = Wallet.Model.JsonRpcResponse.JsonRpcError(
                    id = request.request.id,
                    code = 4001,
                    message = "User rejected the request",
                ),
            ),
            onSuccess = {},
            onError = {},
        )
        onCancel()
    }

    fun rejectSessionProposal(proposal: Wallet.Model.SessionProposal) {
        bridgesRepository.rejectConnection(proposal, onSuccess = {}, onError = {})
        onCancel()
    }

    fun rejectSessionAuthenticate(request: Wallet.Model.SessionAuthenticate) {
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

    class SessionRequest(val request: Wallet.Model.SessionRequest, val verifyContext: Wallet.Model.VerifyContext?) : WalletConnectIntent {
        override val requiresUserAction = true
    }

    class AuthRequest(val request: Wallet.Model.SessionAuthenticate, val verifyContext: Wallet.Model.VerifyContext?) : WalletConnectIntent {
        override val requiresUserAction = true
    }

    class SessionProposal(val sessionProposal: Wallet.Model.SessionProposal, val verifyContext: Wallet.Model.VerifyContext?) : WalletConnectIntent {
        override val requiresUserAction = true
    }

    class ConnectionState(val error: String?) : WalletConnectIntent {
        override val requiresUserAction = false
    }
}

private fun WalletConnectEvent.toUIState(): WalletConnectIntent? {
    return when (val model = model) {
        is Wallet.Model.SessionRequest -> WalletConnectIntent.SessionRequest(model, verifyContext)
        is Wallet.Model.SessionAuthenticate -> WalletConnectIntent.AuthRequest(model, verifyContext)
        is Wallet.Model.SessionProposal -> WalletConnectIntent.SessionProposal(model, verifyContext)
        else -> null
    }
}
