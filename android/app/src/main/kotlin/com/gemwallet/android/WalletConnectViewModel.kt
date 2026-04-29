package com.gemwallet.android

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.bridge.BridgesRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectEvent
import com.reown.walletkit.client.Wallet
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@HiltViewModel
class WalletConnectViewModel @Inject constructor(
    bridgesRepository: BridgesRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow<WalletConnectIntent>(WalletConnectIntent.Idle)
    val uiState = _uiState.asStateFlow()

    init {
        bridgesRepository.bridgeEvents
            .onEach { event ->
                event.toUIState()?.let { intent -> _uiState.update { intent } }
            }
            .launchIn(viewModelScope)
    }

    fun onCancel() {
        _uiState.update { WalletConnectIntent.Idle }
    }
}

sealed interface WalletConnectIntent {

    data object Idle : WalletConnectIntent

    data object Cancel : WalletConnectIntent

    data object SessionDelete : WalletConnectIntent

    class SessionRequest(val request: Wallet.Model.SessionRequest, val verifyContext: Wallet.Model.VerifyContext?) : WalletConnectIntent

    class AuthRequest(val request: Wallet.Model.SessionAuthenticate, val verifyContext: Wallet.Model.VerifyContext?) : WalletConnectIntent

    class SessionProposal(val sessionProposal: Wallet.Model.SessionProposal, val verifyContext: Wallet.Model.VerifyContext?) : WalletConnectIntent

    class ConnectionState(val error: String?) : WalletConnectIntent
}

private fun WalletConnectEvent.toUIState(): WalletConnectIntent? {
    return when (val model = model) {
        is Wallet.Model.SessionRequest -> WalletConnectIntent.SessionRequest(model, verifyContext)
        is Wallet.Model.SessionAuthenticate -> WalletConnectIntent.AuthRequest(model, verifyContext)
        is Wallet.Model.SessionDelete -> WalletConnectIntent.SessionDelete
        is Wallet.Model.SessionProposal -> WalletConnectIntent.SessionProposal(model, verifyContext)
        else -> null
    }
}
