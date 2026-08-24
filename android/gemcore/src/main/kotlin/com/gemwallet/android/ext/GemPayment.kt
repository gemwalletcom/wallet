package com.gemwallet.android.ext

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentAction
import com.wallet.core.primitives.PaymentActionSendInner
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentMerchant
import com.wallet.core.primitives.PaymentOptions
import com.wallet.core.primitives.PaymentOutcome
import com.wallet.core.primitives.PaymentPrice
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuoteData
import com.wallet.core.primitives.PaymentQuotes
import com.wallet.core.primitives.PaymentRequest
import com.wallet.core.primitives.PaymentStatus
import com.wallet.core.primitives.SerializedDate
import uniffi.gemstone.GemPayment
import uniffi.gemstone.GemPaymentAction
import uniffi.gemstone.GemPaymentAmount
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentMerchant
import uniffi.gemstone.GemPaymentOptions
import uniffi.gemstone.GemPaymentOutcome
import uniffi.gemstone.GemPaymentPrice
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuoteData
import uniffi.gemstone.GemPaymentQuotes
import uniffi.gemstone.GemPaymentRequest
import uniffi.gemstone.GemPaymentStatus

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
    is GemPaymentLink.WalletConnectPay -> PaymentLink.WalletConnectPay(v1)
}

fun PaymentLink.toGem(): GemPaymentLink = when (this) {
    is PaymentLink.SolanaPay -> GemPaymentLink.SolanaPay(content)
    is PaymentLink.WalletConnectPay -> GemPaymentLink.WalletConnectPay(content)
}

fun GemPaymentOptions.toPrimitives(): PaymentOptions = when (this) {
    is GemPaymentOptions.Quotes -> PaymentOptions.Quotes(v1.toPrimitives())
    is GemPaymentOptions.Outcome -> PaymentOptions.Outcome(v1.toPrimitives())
}

fun GemPaymentQuotes.toPrimitives(): PaymentQuotes = PaymentQuotes(
    merchant = merchant.toPrimitives(),
    price = price?.toPrimitives(),
    expiresAt = expiresAt?.toSerializedDate(),
    quotes = quotes.mapNotNull { it.toPrimitives() },
)

fun GemPaymentQuote.toPrimitives(): PaymentQuote? = PaymentQuote(
    id = id,
    link = link.toPrimitives(),
    assetId = assetId.toAssetId() ?: return null,
    value = value,
    expiresAt = expiresAt?.toSerializedDate(),
    collectDataUrl = collectDataUrl,
    providerData = providerData,
)

fun PaymentQuote.toGem(): GemPaymentQuote = GemPaymentQuote(
    id = id,
    link = link.toGem(),
    assetId = assetId.toIdentifier(),
    value = value,
    expiresAt = expiresAt?.toDateTimeUtc(),
    collectDataUrl = collectDataUrl,
    providerData = providerData,
)

fun GemPaymentPrice.toPrimitives(): PaymentPrice = PaymentPrice(
    symbol = symbol,
    value = value,
    decimals = decimals,
)

fun GemPaymentMerchant.toPrimitives(): PaymentMerchant = PaymentMerchant(
    name = name,
    iconUrl = iconUrl,
)

fun GemPaymentStatus.toPrimitives(): PaymentStatus = when (this) {
    GemPaymentStatus.REQUIRES_ACTION -> PaymentStatus.RequiresAction
    GemPaymentStatus.PROCESSING -> PaymentStatus.Processing
    GemPaymentStatus.SUCCEEDED -> PaymentStatus.Succeeded
    GemPaymentStatus.FAILED -> PaymentStatus.Failed
    GemPaymentStatus.EXPIRED -> PaymentStatus.Expired
    GemPaymentStatus.CANCELLED -> PaymentStatus.Cancelled
}

fun GemPaymentOutcome.toPrimitives(): PaymentOutcome = PaymentOutcome(
    status = status.toPrimitives(),
    transactionId = transactionId,
)

fun GemPaymentAction.toPrimitives(): PaymentAction? = when (this) {
    is GemPaymentAction.Send -> PaymentAction.Send(
        PaymentActionSendInner(
            chain = chain.toChain() ?: return null,
            recipient = recipient,
            value = value,
            data = data,
        )
    )
}

fun GemPaymentQuoteData.toPrimitives(): PaymentQuoteData? = PaymentQuoteData(
    quote = quote.toPrimitives() ?: return null,
    action = action.toPrimitives() ?: return null,
)

private fun Long.toSerializedDate(): SerializedDate = this * 1_000

private fun SerializedDate.toDateTimeUtc(): Long = this / 1_000
