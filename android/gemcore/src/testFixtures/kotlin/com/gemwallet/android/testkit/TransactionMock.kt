package com.gemwallet.android.testkit

import com.gemwallet.android.model.Transaction
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.TransactionUtxoInput

fun mockTransaction(
    assetId: AssetId = AssetId(chain = com.wallet.core.primitives.Chain.Bitcoin),
    id: TransactionId = mockTransactionId(chain = assetId.chain),
    from: String = "sender-address",
    to: String = "recipient-address",
    contract: String? = null,
    type: TransactionType = TransactionType.Transfer,
    state: TransactionState = TransactionState.Confirmed,
    blockNumber: String? = "1",
    sequence: String? = null,
    fee: String = "1",
    feeAssetId: AssetId = assetId,
    value: String = "1",
    memo: String? = null,
    direction: TransactionDirection = TransactionDirection.Outgoing,
    utxoInputs: List<TransactionUtxoInput>? = null,
    utxoOutputs: List<TransactionUtxoInput>? = null,
    metadata: String? = null,
    createdAt: Long = 1L,
) = Transaction(
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
