package com.gemwallet.android.ext

import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemPaymentException
import uniffi.gemstone.paymentErrorText

val GemPaymentException.errorText: GemErrorText?
    get() = when (this) {
        is GemPaymentException.NoPaymentOptions -> null
        is GemPaymentException.Status, is GemPaymentException.InvalidRequest, is GemPaymentException.Network -> paymentErrorText(this)
    }
