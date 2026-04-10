package com.gemwallet.android.testkit

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId

fun mockTransactionId(
    chain: Chain = Chain.Bitcoin,
    hash: String = "tx-id",
) = TransactionId(
    chain = chain,
    hash = hash,
)
