package com.gemwallet.android.domains.transaction

import uniffi.gemstone.GemAmountSign
import uniffi.gemstone.GemTransactionValue

fun GemAmountSign.format(amount: String): String = when (this) {
    GemAmountSign.INCOMING -> "+$amount"
    GemAmountSign.OUTGOING -> "-$amount"
    GemAmountSign.NONE -> amount
}

fun GemTransactionValue.sign(): GemAmountSign = when (this) {
    is GemTransactionValue.Amount -> sign
    GemTransactionValue.SwapReceived -> GemAmountSign.INCOMING
    GemTransactionValue.SwapSpent -> GemAmountSign.OUTGOING
    GemTransactionValue.AssetSymbol,
    GemTransactionValue.PerpetualNotional,
    is GemTransactionValue.PerpetualPnl,
    GemTransactionValue.None -> GemAmountSign.NONE
}
