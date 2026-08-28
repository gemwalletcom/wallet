package com.gemwallet.android.data.repositories.bridge

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSession
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import uniffi.gemstone.SignMessage as GemSignMessage
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.serializer.decodeJson
import uniffi.gemstone.GemWalletConnectMessageRequest
import uniffi.gemstone.GemWalletConnectSigner
import uniffi.gemstone.GemWalletConnectTransactionAction
import uniffi.gemstone.GemWalletConnectTransactionRequest
import uniffi.gemstone.GemTransferData

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
        val transfer: GemTransferData,
        val isSendable: Boolean,
    ) : WalletConnectPendingRequest(sessionId, chain, wallet, account, appMetadata, simulation)
}

class WalletConnectPendingRequests : GemWalletConnectSigner {
    private val _current = MutableStateFlow<WalletConnectPendingRequest?>(null)
    val current: StateFlow<WalletConnectPendingRequest?> = _current.asStateFlow()

    override suspend fun signMessage(request: GemWalletConnectMessageRequest): String {
        val chain = checkNotNull(request.chain.toChain()) { "Unsupported chain ${request.chain}" }
        val wallet = request.wallet.decodeJson<Wallet>()
        val account = checkNotNull(wallet.getAccount(chain)) { "Wallet has no $chain account" }
        val session = request.session.decodeJson<WalletConnectionSession>()
        return await(
            WalletConnectPendingRequest.SignMessage(
                sessionId = request.sessionId,
                chain = chain,
                wallet = wallet,
                account = account,
                appMetadata = session.metadata,
                simulation = request.simulation.decodeJson(),
                message = request.message,
            ),
        )
    }

    override suspend fun signTransaction(request: GemWalletConnectTransactionRequest): String {
        val chain = checkNotNull(request.chain.toChain()) { "Unsupported chain ${request.chain}" }
        val wallet = request.wallet.decodeJson<Wallet>()
        val account = checkNotNull(wallet.getAccount(chain)) { "Wallet has no $chain account" }
        val session = request.session.decodeJson<WalletConnectionSession>()
        return await(
            WalletConnectPendingRequest.Transaction(
                sessionId = request.sessionId,
                chain = chain,
                wallet = wallet,
                account = account,
                appMetadata = session.metadata,
                simulation = request.simulation.decodeJson(),
                transfer = request.transfer,
                isSendable = request.action == GemWalletConnectTransactionAction.SEND,
            ),
        )
    }

    suspend fun await(request: WalletConnectPendingRequest): String {
        _current.value = request
        try {
            return request.result.await()
        } finally {
            _current.compareAndSet(request, null)
        }
    }
}
