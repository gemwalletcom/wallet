package com.gemwallet.android.testkit

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType

fun mockTransaction(
    assetId: AssetId = mockAssetId(),
    id: TransactionId = mockTransactionId(chain = assetId.chain),
    from: String = "sender-address",
    to: String = "recipient-address",
    type: TransactionType = TransactionType.Transfer,
    state: TransactionState = TransactionState.Confirmed,
    feeAssetId: AssetId = assetId,
    value: String = "1",
    direction: TransactionDirection = TransactionDirection.Outgoing,
    metadata: String? = null,
    createdAt: Long = 1L,
) = Transaction(
    id = id,
    assetId = assetId,
    from = from,
    to = to,
    contract = null,
    type = type,
    state = state,
    blockNumber = "1",
    sequence = null,
    fee = "1",
    feeAssetId = feeAssetId,
    value = value,
    memo = null,
    direction = direction,
    utxoInputs = null,
    utxoOutputs = null,
    metadata = metadata,
    createdAt = createdAt,
)
