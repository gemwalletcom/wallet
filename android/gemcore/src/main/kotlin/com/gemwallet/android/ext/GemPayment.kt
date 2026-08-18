package com.gemwallet.android.ext

import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.GemPayment
import uniffi.gemstone.GemPaymentAmount
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentRequest

fun GemPayment.toPrimitives(): Payment = when (this) {
    is GemPayment.Request -> Payment.Request(v1.toPrimitives())
    is GemPayment.Link -> Payment.Link(v1.toPrimitives())
}

fun GemPaymentRequest.toPrimitives(): PaymentRequest = PaymentRequest(
    address = address,
    amount = amount?.toPrimitives(),
    memo = memo,
    assetId = assetId?.toAssetId(),
)

fun GemPaymentAmount.toPrimitives(): PaymentAmount = when (this) {
    is GemPaymentAmount.ExactValue -> PaymentAmount.ExactValue(v1)
    is GemPaymentAmount.AtomicValue -> PaymentAmount.AtomicValue(v1)
}

fun GemPaymentLink.toPrimitives(): PaymentLink = when (this) {
    is GemPaymentLink.SolanaPay -> PaymentLink.SolanaPay(v1)
}
