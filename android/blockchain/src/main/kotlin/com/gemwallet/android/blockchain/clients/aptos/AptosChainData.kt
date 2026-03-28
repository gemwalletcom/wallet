package com.gemwallet.android.blockchain.clients.aptos

import com.gemwallet.android.model.ChainSignData
import uniffi.gemstone.GemTransactionLoadMetadata

data class AptosChainData(
    val sequence: ULong,
    val gasLimit: ULong?,
    val data: String?,
) : ChainSignData {
    override fun toDto(): GemTransactionLoadMetadata = GemTransactionLoadMetadata.Aptos(
        sequence = sequence,
        gasLimit = gasLimit,
        data = data,
    )
}

fun GemTransactionLoadMetadata.Aptos.toChainData(): AptosChainData {
    return AptosChainData(
        sequence = sequence,
        gasLimit = gasLimit,
        data = data,
    )
}
