package com.gemwallet.android.ext

import com.wallet.core.primitives.TransactionState

fun TransactionState.isCompleted(): Boolean = when (this) {
    TransactionState.Confirmed,
    TransactionState.Failed,
    TransactionState.Reverted -> true
    TransactionState.Pending,
    TransactionState.InTransit -> false
}
