package com.gemwallet.android.ext

import uniffi.gemstone.GemPaymentException

val GemPaymentException.userMessage: String?
    get() = when (this) {
        is GemPaymentException.InvalidRequest -> reason
        is GemPaymentException.Network -> reason
        is GemPaymentException.NoPaymentOptions -> null
    }
