package com.gemwallet.android.ext

import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.serializer.unpackRoutePayload
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.paymentDecodeUrl

val Payment.request: PaymentRequest?
    get() = when (this) {
        is Payment.Request -> content
        is Payment.Link -> null
    }

fun decodePayment(url: String): Payment? =
    runCatching { paymentDecodeUrl(url) }.getOrNull()?.toPrimitives()

fun PaymentRequest.pack(): String? = packRoutePayload()

fun unpackPaymentRequest(input: String): PaymentRequest? = unpackRoutePayload(input)
