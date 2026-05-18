package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toGem
import com.gemwallet.android.blockchain.gemstone.toPrimitives
import com.gemwallet.android.blockchain.model.ServiceUnavailable
import com.gemwallet.android.model.TransactionChanges
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionStateRequest
import com.wallet.core.primitives.TransactionSwapStateRequest
import uniffi.gemstone.GemGatewayInterface

class TransactionStatusService(
    private val gateway: GemGatewayInterface,
) {
    suspend fun getStatus(chain: Chain, request: TransactionStateRequest): TransactionChanges {
        return try {
            gateway.getTransactionStatus(
                chain = chain.string,
                request.toGem(),
            ).toPrimitives()
        } catch (_: Throwable) {
            throw ServiceUnavailable
        }
    }

    suspend fun getSwapStatus(chain: Chain, request: TransactionSwapStateRequest): TransactionChanges {
        return try {
            gateway.getTransactionSwapStatus(
                chain = chain.string,
                request = request.toGem(),
            ).toPrimitives()
        } catch (_: Throwable) {
            throw ServiceUnavailable
        }
    }
}
