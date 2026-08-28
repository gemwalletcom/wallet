package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequests
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnectionSession
import uniffi.gemstone.GemWalletConnectSignPayload
import uniffi.gemstone.GemWalletConnectSignRequest
import uniffi.gemstone.GemWalletConnectSigner
import uniffi.gemstone.GemWalletConnectTransactionAction

class GemstoneWalletConnectSigner(
    private val pendingRequests: WalletConnectPendingRequests,
) : GemWalletConnectSigner {
    override suspend fun sign(request: GemWalletConnectSignRequest): String {
        val chain = checkNotNull(request.chain.toChain()) { "Unsupported chain ${request.chain}" }
        val wallet = request.wallet.decodeJson<Wallet>()
        val account = checkNotNull(wallet.getAccount(chain)) { "Wallet has no $chain account" }
        val session = request.session.decodeJson<WalletConnectionSession>()
        val pending = when (val payload = request.payload) {
            is GemWalletConnectSignPayload.Message -> WalletConnectPendingRequest.SignMessage(
                sessionId = request.sessionId,
                chain = chain,
                wallet = wallet,
                account = account,
                appMetadata = session.metadata,
                simulation = request.simulation.decodeJson(),
                message = payload.message,
            )
            is GemWalletConnectSignPayload.Transaction -> WalletConnectPendingRequest.Transaction(
                sessionId = request.sessionId,
                chain = chain,
                wallet = wallet,
                account = account,
                appMetadata = session.metadata,
                simulation = request.simulation.decodeJson(),
                transfer = payload.transfer,
                isSendable = payload.action == GemWalletConnectTransactionAction.SEND,
            )
        }
        return pendingRequests.await(pending)
    }
}
