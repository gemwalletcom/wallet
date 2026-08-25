package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.SimulationResult
import uniffi.gemstone.SignDigestType
import uniffi.gemstone.TransactionSimulationServiceInterface
import uniffi.gemstone.WalletConnectTransactionType

class TransactionSimulationService(
    private val service: TransactionSimulationServiceInterface,
) {
    suspend fun simulateSignMessage(chain: String, signType: SignDigestType, data: String, sessionDomain: String): SimulationResult =
        service.simulateSignMessage(chain = chain, signType = signType, data = data, sessionDomain = sessionDomain).toPrimitives()

    suspend fun simulateSendTransaction(chain: String, transactionType: WalletConnectTransactionType, data: String): SimulationResult =
        service.simulateSendTransaction(chain = chain, transactionType = transactionType, data = data).toPrimitives()

    suspend fun simulateTransaction(chain: Chain, encodedTransaction: String, signerAddress: String?): SimulationResult =
        service.simulateTransaction(chain = chain.string, encodedTransaction = encodedTransaction, signerAddress = signerAddress).toPrimitives()
}
