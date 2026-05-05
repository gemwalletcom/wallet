package com.gemwallet.android.ext

import com.gemwallet.android.model.Transaction
import com.wallet.core.primitives.Transaction as CoreTransaction

fun CoreTransaction.toModel(): Transaction {
    return Transaction(
        id = id,
        assetId = assetId,
        from = from,
        to = to,
        contract = contract,
        type = type,
        state = state,
        blockNumber = blockNumber,
        sequence = sequence,
        fee = fee,
        feeAssetId = feeAssetId,
        value = value,
        memo = memo,
        direction = direction,
        utxoInputs = utxoInputs,
        utxoOutputs = utxoOutputs,
        metadata = metadata,
        createdAt = createdAt,
    )
}
