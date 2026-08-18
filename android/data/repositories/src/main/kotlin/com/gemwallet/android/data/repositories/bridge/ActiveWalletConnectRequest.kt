package com.gemwallet.android.data.repositories.bridge

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.update

class ActiveWalletConnectRequest(
    events: Flow<WalletConnectEvent>,
    scope: CoroutineScope = CoroutineScope(Dispatchers.Main.immediate + SupervisorJob()),
) {

    private val _current = MutableStateFlow<WalletConnectUserRequest?>(null)
    val current: StateFlow<WalletConnectUserRequest?> = _current.asStateFlow()

    init {
        events.onEach { event ->
            val request = event.toUserRequest() ?: return@onEach
            _current.update { request }
        }.launchIn(scope)
    }

    fun finish() {
        _current.update { null }
    }
}

sealed interface WalletConnectUserRequest {

    val verifyContext: WalletConnectVerifyContext

    class SessionRequest(
        val request: WalletConnectSessionRequest,
        override val verifyContext: WalletConnectVerifyContext,
    ) : WalletConnectUserRequest

    class AuthenticationRequest(
        val request: WalletConnectAuthenticationRequest,
        override val verifyContext: WalletConnectVerifyContext,
    ) : WalletConnectUserRequest

    class SessionProposal(
        val proposal: WalletConnectSessionProposal,
        override val verifyContext: WalletConnectVerifyContext,
    ) : WalletConnectUserRequest
}

private fun WalletConnectEvent.toUserRequest(): WalletConnectUserRequest? = when (this) {
    is WalletConnectEvent.SessionRequest -> WalletConnectUserRequest.SessionRequest(request, verifyContext)
    is WalletConnectEvent.AuthenticationRequest -> WalletConnectUserRequest.AuthenticationRequest(request, verifyContext)
    is WalletConnectEvent.SessionProposal -> WalletConnectUserRequest.SessionProposal(proposal, verifyContext)
    is WalletConnectEvent.SessionSettled,
    is WalletConnectEvent.SessionDeleted -> null
}
