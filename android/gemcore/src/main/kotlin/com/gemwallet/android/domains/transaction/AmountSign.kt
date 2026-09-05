package com.gemwallet.android.domains.transaction

import uniffi.gemstone.GemAmountSign

fun GemAmountSign.format(amount: String): String = when (this) {
    GemAmountSign.INCOMING -> "+$amount"
    GemAmountSign.OUTGOING -> "-$amount"
    GemAmountSign.NONE -> amount
}
