package com.gemwallet.android.data.repositories.bridge

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.gemstone.SignMessage as GemSignMessage
import uniffi.gemstone.WalletConnectTransaction

class WalletConnectRequestRejected : Exception("User rejected the request")

sealed class WalletConnectPendingRequest(
    val sessionId: String,
    val chain: Chain,
    val wallet: Wallet,
    val account: Account,
    val appMetadata: ApplicationMetadata,
    val simulation: SimulationResult,
) {
    internal val result = CompletableDeferred<String>()

    fun approve(value: String) {
        result.complete(value)
    }

    fun reject() {
        result.completeExceptionally(WalletConnectRequestRejected())
    }

    class SignMessage(
        sessionId: String,
        chain: Chain,
        wallet: Wallet,
        account: Account,
        appMetadata: ApplicationMetadata,
        simulation: SimulationResult,
        val message: GemSignMessage,
    ) : WalletConnectPendingRequest(sessionId, chain, wallet, account, appMetadata, simulation)

    class Transaction(
        sessionId: String,
        chain: Chain,
        wallet: Wallet,
        account: Account,
        appMetadata: ApplicationMetadata,
        simulation: SimulationResult,
        val transaction: WalletConnectTransaction,
        val isSendable: Boolean,
    ) : WalletConnectPendingRequest(sessionId, chain, wallet, account, appMetadata, simulation)
}

class WalletConnectPendingRequests {
    private val _current = MutableStateFlow<WalletConnectPendingRequest?>(null)
    val current: StateFlow<WalletConnectPendingRequest?> = _current.asStateFlow()

    suspend fun await(request: WalletConnectPendingRequest): String {
        _current.value = request
        try {
            return request.result.await()
        } finally {
            _current.compareAndSet(request, null)
        }
    }
}
