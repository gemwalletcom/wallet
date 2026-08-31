package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.serializer.decodeJson
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
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemWalletConnectMessageRequest
import uniffi.gemstone.GemWalletConnectSigner
import uniffi.gemstone.GemWalletConnectTransactionAction
import uniffi.gemstone.GemWalletConnectTransactionRequest
import uniffi.gemstone.SignMessage as GemSignMessage

sealed class WalletConnectPendingRequest(
    val sessionId: String,
    chainId: String,
    walletJson: String,
    sessionJson: String,
    simulationJson: String,
) {
    internal val result = CompletableDeferred<String>()

    val chain: Chain by lazy { checkNotNull(chainId.toChain()) { "Unsupported chain $chainId" } }
    val wallet: Wallet by lazy { walletJson.decodeJson() }
    val account: Account by lazy { checkNotNull(wallet.getAccount(chain)) { "Wallet has no $chain account" } }
    val appMetadata: ApplicationMetadata by lazy { sessionJson.decodeJson<WalletConnectionSession>().metadata }
    val simulation: SimulationResult by lazy { simulationJson.decodeJson() }

    fun approve(value: String) {
        result.complete(value)
    }

    fun reject() {
        result.completeExceptionally(GemServiceException.Cancelled())
    }

    class SignMessage(
        private val request: GemWalletConnectMessageRequest,
    ) : WalletConnectPendingRequest(request.sessionId, request.chain, request.wallet, request.session, request.simulation) {
        val message: GemSignMessage get() = request.message
    }

    class Transaction(
        private val request: GemWalletConnectTransactionRequest,
    ) : WalletConnectPendingRequest(request.sessionId, request.chain, request.wallet, request.session, request.simulation) {
        val transfer: GemTransferData get() = request.transfer
        val isSendable: Boolean get() = request.action == GemWalletConnectTransactionAction.SEND
    }
}

class WalletConnectPendingRequests : GemWalletConnectSigner {
    private val _current = MutableStateFlow<WalletConnectPendingRequest?>(null)
    val current: StateFlow<WalletConnectPendingRequest?> = _current.asStateFlow()

    override suspend fun signMessage(request: GemWalletConnectMessageRequest): String = await(WalletConnectPendingRequest.SignMessage(request))

    override suspend fun signTransaction(request: GemWalletConnectTransactionRequest): String = await(WalletConnectPendingRequest.Transaction(request))

    private suspend fun await(request: WalletConnectPendingRequest): String {
        _current.value = request
        try {
            return request.result.await()
        } finally {
            _current.compareAndSet(request, null)
        }
    }
}
