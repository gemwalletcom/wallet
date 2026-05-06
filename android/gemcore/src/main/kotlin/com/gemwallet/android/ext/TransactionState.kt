package com.gemwallet.android.ext

import com.wallet.core.primitives.TransactionState

fun TransactionState.isComplete(): Boolean = when (this) {
    TransactionState.Confirmed,
    TransactionState.Failed,
    TransactionState.Reverted -> true
    TransactionState.Pending,
    TransactionState.InTransit -> false
}
