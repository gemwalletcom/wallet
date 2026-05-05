package com.gemwallet.android.domains.transaction

import com.wallet.core.primitives.TransactionType

val TransactionType.isPerpetual: Boolean
    get() = when (this) {
        TransactionType.PerpetualOpenPosition,
        TransactionType.PerpetualClosePosition,
        TransactionType.PerpetualModifyPosition -> true
        else -> false
    }
