package com.gemwallet.android.application.wallet_connect

import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.getAndUpdate
import uniffi.gemstone.Asset
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemWalletConnectMessageRequest
import uniffi.gemstone.GemWalletConnectSigner
import uniffi.gemstone.GemWalletConnectTransactionRequest
import uniffi.gemstone.SimulationResult
import java.util.UUID
import uniffi.gemstone.SignMessage as GemSignMessage

sealed class WalletConnectPendingRequest(val sessionId: String, val requestId: String, chainId: String, val wallet: Wallet, session: uniffi.gemstone.WalletConnectionSession, val simulation: SimulationResult) {
    internal val result = CompletableDeferred<String>()
    val reviewId: String = UUID.randomUUID().toString()

    val chain: Chain by lazy { chainId.requireChain() }
    val account: Account by lazy { checkNotNull(wallet.getAccount(chain)) { "Wallet has no $chain account" } }
    val appMetadata: ApplicationMetadata = session.metadata.toPrimitives()

    fun approve(value: String) {
        result.complete(value)
    }

    fun reject() {
        result.completeExceptionally(GemServiceException.Cancelled())
    }

    fun expire() {
        result.completeExceptionally(GemServiceException.Expired())
    }

    class SignMessage(val request: GemWalletConnectMessageRequest) : WalletConnectPendingRequest(request.sessionId, request.requestId, request.chain, request.wallet.toPrimitives(), request.session, request.simulation) {
        val message: GemSignMessage get() = request.message
        val assets: List<Asset> get() = request.assets
    }

    class Transaction(private val request: GemWalletConnectTransactionRequest) : WalletConnectPendingRequest(request.sessionId, request.requestId, request.chain, request.wallet.toPrimitives(), request.session, request.simulation) {
        val transfer: GemTransferData get() = request.transfer
    }
}

class WalletConnectPendingRequests : GemWalletConnectSigner {
    private val _current = MutableStateFlow<WalletConnectPendingRequest?>(null)
    val current: StateFlow<WalletConnectPendingRequest?> = _current.asStateFlow()

    override suspend fun signMessage(request: GemWalletConnectMessageRequest): String = await(WalletConnectPendingRequest.SignMessage(request))

    override suspend fun signTransaction(request: GemWalletConnectTransactionRequest): String = await(WalletConnectPendingRequest.Transaction(request))

    fun expire(requestId: String): Boolean {
        val request = _current.value?.takeIf { it.requestId == requestId } ?: return false
        request.expire()
        return true
    }

    private suspend fun await(request: WalletConnectPendingRequest): String {
        _current.getAndUpdate { request }?.reject()
        try {
            return request.result.await()
        } finally {
            _current.compareAndSet(request, null)
        }
    }
}
