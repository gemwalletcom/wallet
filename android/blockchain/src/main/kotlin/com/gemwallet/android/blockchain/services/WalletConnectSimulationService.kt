package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.wallet.core.primitives.SimulationResult
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.WalletConnectSimulationClientInterface
import uniffi.gemstone.WalletConnectTransactionType

class WalletConnectSimulationService(
    private val client: WalletConnectSimulationClientInterface,
) {
    suspend fun simulateSignMessage(chain: String, signType: SignDigestType, data: String, sessionDomain: String): SimulationResult =
        client.simulateSignMessage(chain = chain, signType = signType, data = data, sessionDomain = sessionDomain).toPrimitives()

    suspend fun simulateSendTransaction(chain: String, transactionType: WalletConnectTransactionType, data: String): SimulationResult =
        client.simulateSendTransaction(chain = chain, transactionType = transactionType, data = data).toPrimitives()
}
