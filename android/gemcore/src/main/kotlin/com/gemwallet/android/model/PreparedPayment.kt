package com.gemwallet.android.model

import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
import uniffi.gemstone.PaymentAction

data class PreparedPayment(
    val quotes: PaymentQuotes,
    val quote: PaymentQuote,
    val actions: List<PaymentAction>,
    val isRelayed: Boolean,
)
