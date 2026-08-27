package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.bridge.ConnectionsRepository
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequest
import com.gemwallet.android.data.repositories.bridge.WalletConnectPendingRequests
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnection
import uniffi.gemstone.GemWalletConnectSigner
import uniffi.gemstone.SignMessage
import uniffi.gemstone.WalletConnectTransaction
import uniffi.gemstone.Account as GemAccount

class GemstoneWalletConnectSigner(
    private val connectionsRepository: ConnectionsRepository,
    private val pendingRequests: WalletConnectPendingRequests,
) : GemWalletConnectSigner {
    override suspend fun signMessage(sessionId: String, chain: String, message: SignMessage, simulation: String): String {
        val session = resolve(sessionId, chain)
        return pendingRequests.await(
            WalletConnectPendingRequest.SignMessage(
                sessionId = sessionId,
                chain = session.chain,
                wallet = session.connection.wallet,
                account = session.account,
                appMetadata = session.connection.session.metadata,
                simulation = simulation.decodeJson(),
                message = message,
            ),
        )
    }

    override suspend fun signTransaction(sessionId: String, chain: String, transaction: WalletConnectTransaction, simulation: String): String =
        awaitTransaction(sessionId, chain, transaction, simulation, isSendable = false)

    override suspend fun sendTransaction(sessionId: String, chain: String, transaction: WalletConnectTransaction, simulation: String): String =
        awaitTransaction(sessionId, chain, transaction, simulation, isSendable = true)

    override suspend fun getAccounts(sessionId: String, chain: String): List<GemAccount> {
        val session = resolve(sessionId, chain)
        return session.connection.wallet.accounts.filter { it.chain == session.chain }.map { it.toGem() }
    }

    private suspend fun awaitTransaction(
        sessionId: String,
        chain: String,
        transaction: WalletConnectTransaction,
        simulation: String,
        isSendable: Boolean,
    ): String {
        val session = resolve(sessionId, chain)
        return pendingRequests.await(
            WalletConnectPendingRequest.Transaction(
                sessionId = sessionId,
                chain = session.chain,
                wallet = session.connection.wallet,
                account = session.account,
                appMetadata = session.connection.session.metadata,
                simulation = simulation.decodeJson(),
                transaction = transaction,
                isSendable = isSendable,
            ),
        )
    }

    private suspend fun resolve(sessionId: String, chain: String): ResolvedSession {
        val connection = checkNotNull(connectionsRepository.getConnectionByTopic(sessionId)) { "Unknown WalletConnect session" }
        val resolvedChain = checkNotNull(chain.toChain()) { "Unsupported chain $chain" }
        check(connection.session.chains.contains(resolvedChain)) { "Chain $chain is not part of the session" }
        val account = checkNotNull(connection.wallet.getAccount(resolvedChain)) { "Wallet has no $chain account" }
        return ResolvedSession(connection, resolvedChain, account)
    }

    private class ResolvedSession(
        val connection: WalletConnection,
        val chain: Chain,
        val account: Account,
    )
}
