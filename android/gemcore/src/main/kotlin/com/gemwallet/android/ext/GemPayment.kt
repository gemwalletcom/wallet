package com.gemwallet.android.ext

import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentLinkSolanaPayInner
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.GemPayment
import uniffi.gemstone.GemPaymentAmount
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentRequest
import uniffi.gemstone.GemPaymentException

fun GemPayment.toPrimitives(): Payment = when (this) {
    is GemPayment.Request -> Payment.Request(v1.toPrimitives())
    is GemPayment.Link -> Payment.Link(v1.toPrimitives())
}

fun GemPaymentRequest.toPrimitives(): PaymentRequest = PaymentRequest(
    address = address,
    amount = amount?.toPrimitives(),
    memo = memo,
    references = references,
    assetId = assetId?.toAssetId(),
)

fun GemPaymentAmount.toPrimitives(): PaymentAmount = when (this) {
    is GemPaymentAmount.ExactValue -> PaymentAmount.ExactValue(v1)
    is GemPaymentAmount.AtomicValue -> PaymentAmount.AtomicValue(v1)
}

fun GemPaymentLink.toPrimitives(): PaymentLink = when (this) {
    is GemPaymentLink.SolanaPay -> PaymentLink.SolanaPay(PaymentLinkSolanaPayInner(url))
}

fun PaymentRequest.toGem(): GemPaymentRequest = GemPaymentRequest(
    address = address,
    amount = amount?.toGem(),
    memo = memo,
    references = references,
    assetId = assetId?.toIdentifier(),
)

fun PaymentAmount.toGem(): GemPaymentAmount = when (this) {
    is PaymentAmount.ExactValue -> GemPaymentAmount.ExactValue(content)
    is PaymentAmount.AtomicValue -> GemPaymentAmount.AtomicValue(content)
}

fun PaymentLink.toGem(): GemPaymentLink = when (this) {
    is PaymentLink.SolanaPay -> GemPaymentLink.SolanaPay(content.url)
}

val GemPaymentException.userMessage: String?
    get() = when (this) {
        is GemPaymentException.InvalidRequest -> reason
        is GemPaymentException.Network -> reason
        is GemPaymentException.NoPaymentOptions -> null
    }
