package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.wallet.core.primitives.SimulationResult
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.SignMessage
import uniffi.gemstone.WalletConnectSimulationClientInterface
import uniffi.gemstone.SignableTransactionType

class WalletConnectSimulationService(
    private val client: WalletConnectSimulationClientInterface,
) {
    suspend fun simulateSignMessage(chain: String, signType: SignDigestType, data: String, sessionDomain: String): SimulationResult =
        client.simulateSignMessage(chain = chain, signType = signType, data = data, sessionDomain = sessionDomain).toPrimitives()

    suspend fun simulateSignMessage(message: SignMessage, sessionDomain: String): SimulationResult =
        simulateSignMessage(
            chain = message.chain,
            signType = message.signType,
            data = String(message.data, Charsets.UTF_8),
            sessionDomain = sessionDomain,
        )

    suspend fun simulateSendTransaction(chain: String, transactionType: SignableTransactionType, data: String): SimulationResult =
        client.simulateSendTransaction(chain = chain, transactionType = transactionType, data = data).toPrimitives()
}
