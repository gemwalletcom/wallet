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
import uniffi.gemstone.GemWalletConnectAuthentication
import uniffi.gemstone.GemWalletConnectEvent
import uniffi.gemstone.GemWalletConnectProposal
import uniffi.gemstone.GemWalletConnectSessionRequest

class ActiveWalletConnectRequest(events: Flow<GemWalletConnectEvent>, pendingRequests: WalletConnectPendingRequests, scope: CoroutineScope = CoroutineScope(Dispatchers.Main.immediate + SupervisorJob())) {

    private val _current = MutableStateFlow<WalletConnectUserRequest?>(null)
    val current: StateFlow<WalletConnectUserRequest?> = _current.asStateFlow()

    init {
        events.onEach { event ->
            when (event) {
                is GemWalletConnectEvent.SessionDeleted -> _current.update { current -> current?.takeUnless { it is WalletConnectUserRequest.SessionRequest && it.request.topic == event.topic } }

                is GemWalletConnectEvent.RequestExpired -> if (!pendingRequests.expire(event.requestId)) {
                    _current.update { current -> current?.takeUnless { it.isRequest(event) } }
                }

                is GemWalletConnectEvent.Proposal -> _current.update { WalletConnectUserRequest.SessionProposal(event.proposal) }

                is GemWalletConnectEvent.Authentication -> _current.update { WalletConnectUserRequest.AuthenticationRequest(event.authentication) }

                is GemWalletConnectEvent.Request -> _current.update { WalletConnectUserRequest.SessionRequest(event.request) }
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

private fun WalletConnectUserRequest.isRequest(expired: GemWalletConnectEvent.RequestExpired): Boolean = this is WalletConnectUserRequest.SessionRequest && request.topic == expired.topic && request.requestId == expired.requestId

private val WalletConnectUserRequest.payload: Any
    get() = when (this) {
        is WalletConnectUserRequest.SessionRequest -> request
        is WalletConnectUserRequest.AuthenticationRequest -> authentication
        is WalletConnectUserRequest.SessionProposal -> proposal
    }

sealed interface WalletConnectUserRequest {

    val key: String
        get() = when (this) {
            is SessionRequest -> "request/${request.topic}/${request.requestId}"
            is AuthenticationRequest -> "auth/${authentication.id}"
            is SessionProposal -> "proposal/${proposal.id}"
        }

    class SessionRequest(val request: GemWalletConnectSessionRequest) : WalletConnectUserRequest

    class AuthenticationRequest(val authentication: GemWalletConnectAuthentication) : WalletConnectUserRequest

    class SessionProposal(val proposal: GemWalletConnectProposal) : WalletConnectUserRequest
}
