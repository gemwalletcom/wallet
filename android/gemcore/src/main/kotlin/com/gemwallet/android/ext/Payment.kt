package com.gemwallet.android.ext

import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.GemPaymentService

val Payment.request: PaymentRequest?
    get() = when (this) {
        is Payment.Request -> content
        is Payment.Link -> null
    }

fun GemPaymentService.decodePayment(url: String): Payment? =
    runCatching { decodeUrl(url) }.getOrNull()?.decodeJson()
