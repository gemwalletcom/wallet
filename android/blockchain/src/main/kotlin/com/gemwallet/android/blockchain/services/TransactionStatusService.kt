package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.model.TransactionChanges
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Transaction
import uniffi.gemstone.GemGatewayInterface

class TransactionStatusService(
    private val gateway: GemGatewayInterface,
) {
    suspend fun getUpdate(transaction: Transaction): TransactionChanges {
        return try {
            gateway.getTransactionUpdate(transaction.toJson()).toPrimitives()
        } catch (_: Throwable) {
            throw ServiceUnavailable
        }
    }
}
