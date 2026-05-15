package com.gemwallet.android.domains.transaction

import com.wallet.core.primitives.TransactionDirection

enum class AmountSign {
    None, Incoming, Outgoing;

    fun format(amount: String): String = when (this) {
        Incoming -> "+$amount"
        Outgoing -> "-$amount"
        None -> amount
    }

    companion object {
        operator fun invoke(direction: TransactionDirection): AmountSign = when (direction) {
            TransactionDirection.Incoming -> Incoming
            TransactionDirection.Outgoing -> Outgoing
            TransactionDirection.SelfTransfer -> None
        }
    }
}
