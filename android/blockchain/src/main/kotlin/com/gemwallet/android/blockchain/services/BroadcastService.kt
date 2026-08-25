package com.gemwallet.android.blockchain.services

import com.wallet.core.primitives.Account
import uniffi.gemstone.BroadcastOptions
import uniffi.gemstone.GemGateway

class BroadcastService(
    private val gateway: GemGateway,
) {

    suspend fun send(
        account: Account,
        signedMessage: ByteArray,
        options: BroadcastOptions,
    ): String {
        return gateway.transactionBroadcast(
            chain = account.chain.string,
            data = String(signedMessage),
            options = options,
        )
    }
}