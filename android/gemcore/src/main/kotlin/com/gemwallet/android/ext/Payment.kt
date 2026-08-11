package com.gemwallet.android.ext

import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentRequest

val Payment.request: PaymentRequest?
    get() = when (this) {
        is Payment.Request -> content
        is Payment.Link -> null
    }
