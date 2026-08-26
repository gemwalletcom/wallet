package com.gemwallet.android.ext

import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.paymentDecodeUrl

val Payment.request: PaymentRequest?
    get() = when (this) {
        is Payment.Request -> content
        is Payment.Link -> null
    }

val PaymentRequest.exactAmount: String?
    get() = when (val amount = amount) {
        is PaymentAmount.ExactValue -> amount.content
        is PaymentAmount.AtomicValue, null -> null
    }

fun decodePayment(url: String): Payment? =
    runCatching { paymentDecodeUrl(url) }.getOrNull()?.decodeJson()
