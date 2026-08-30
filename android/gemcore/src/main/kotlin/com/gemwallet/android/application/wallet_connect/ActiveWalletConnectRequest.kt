package com.gemwallet.android.application.wallet_connect

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.getAndUpdate
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
            when (event) {
                is WalletConnectEvent.SessionDeleted -> _current.update { null }
                else -> event.toUserRequest()?.let { request -> _current.update { request } }
            }
        }.launchIn(scope)
    }

    fun finish() {
        _current.update { null }
    }

    fun finish(payload: Any): Boolean {
        val previous = _current.getAndUpdate { current ->
            if (current?.payload === payload) null else current
        }
        return previous?.payload === payload
    }
}

private val WalletConnectUserRequest.payload: Any
    get() = when (this) {
        is WalletConnectUserRequest.SessionRequest -> request
        is WalletConnectUserRequest.AuthenticationRequest -> request
        is WalletConnectUserRequest.SessionProposal -> proposal
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
    is WalletConnectEvent.SessionDeleted -> null
}
