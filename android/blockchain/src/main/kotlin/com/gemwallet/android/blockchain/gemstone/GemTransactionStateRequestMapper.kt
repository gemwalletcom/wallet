package com.gemwallet.android.blockchain.gemstone

import com.wallet.core.primitives.TransactionStateRequest
import com.wallet.core.primitives.TransactionSwapStateRequest
import uniffi.gemstone.GemTransactionStateRequest
import uniffi.gemstone.GemTransactionSwapStateRequest

internal fun TransactionStateRequest.toGem() = GemTransactionStateRequest(
    id = id,
    senderAddress = senderAddress,
    createdAt = createdAt / 1_000,
    blockNumber = blockNumber.toULong(),
)

internal fun TransactionSwapStateRequest.toGem() = GemTransactionSwapStateRequest(
    transaction = transaction.toGem(),
    state = state.toGem(),
    swapProvider = swapProvider.toGem(),
    destinationChain = destinationChain.string,
)
