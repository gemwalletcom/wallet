package com.gemwallet.android.ext

import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentAmount
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentMerchant
import com.wallet.core.primitives.PaymentOptions
import com.wallet.core.primitives.PaymentOutcome
import com.wallet.core.primitives.PaymentPrice
import com.wallet.core.primitives.PaymentProviderName
import com.wallet.core.primitives.PaymentQuote
import com.wallet.core.primitives.PaymentQuotes
import com.wallet.core.primitives.PaymentRequest
import com.wallet.core.primitives.PaymentStatus
import com.wallet.core.primitives.TransactionAppMetadata
import uniffi.gemstone.GemPayment
import uniffi.gemstone.GemPaymentAmount
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentMerchant
import uniffi.gemstone.GemPaymentOptions
import uniffi.gemstone.GemPaymentOutcome
import uniffi.gemstone.GemPaymentPrice
import uniffi.gemstone.GemPaymentProviderName
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuotes
import uniffi.gemstone.GemPaymentRequest
import uniffi.gemstone.GemPaymentStatus
import uniffi.gemstone.paymentWalletConnectUrl

fun GemPayment.toPrimitives(): Payment = when (this) {
    is GemPayment.Request -> Payment.Request(v1.toPrimitives())
    is GemPayment.Link -> Payment.Link(v1.toPrimitives())
}

fun GemPaymentRequest.toPrimitives(): PaymentRequest = PaymentRequest(
    address = address,
    amount = amount,
    memo = memo,
    assetId = assetId?.toAssetId(),
)

fun GemPaymentLink.toPrimitives(): PaymentLink = PaymentLink(
    provider = provider.toPrimitives(),
    id = id,
)

fun PaymentLink.toGem(): GemPaymentLink = GemPaymentLink(
    provider = provider.toGem(),
    id = id,
)

fun GemPaymentMerchant.toPrimitives(): PaymentMerchant = PaymentMerchant(
    name = name,
    iconUrl = iconUrl,
)

fun PaymentMerchant.toGem(): GemPaymentMerchant = GemPaymentMerchant(
    name = name,
    iconUrl = iconUrl,
)

fun PaymentMerchant.toAppMetadata(): TransactionAppMetadata = TransactionAppMetadata(
    name = name,
    description = null,
    url = paymentWalletConnectUrl(),
    icon = iconUrl,
)

fun GemPaymentProviderName.toPrimitives(): PaymentProviderName = when (this) {
    GemPaymentProviderName.SOLANA_PAY -> PaymentProviderName.SolanaPay
    GemPaymentProviderName.WALLET_CONNECT_PAY -> PaymentProviderName.WalletConnectPay
}

fun PaymentProviderName.toGem(): GemPaymentProviderName = when (this) {
    PaymentProviderName.SolanaPay -> GemPaymentProviderName.SOLANA_PAY
    PaymentProviderName.WalletConnectPay -> GemPaymentProviderName.WALLET_CONNECT_PAY
}

fun GemPaymentAmount.toPrimitives(): PaymentAmount = PaymentAmount(
    assetId = requireNotNull(assetId.toAssetId()) { "unknown payment asset $assetId" },
    value = value,
    symbol = symbol,
    decimals = decimals,
)

fun PaymentAmount.toGem(): GemPaymentAmount = GemPaymentAmount(
    assetId = assetId.toIdentifier(),
    value = value,
    symbol = symbol,
    decimals = decimals,
)

fun GemPaymentPrice.toPrimitives(): PaymentPrice = PaymentPrice(
    symbol = symbol,
    value = value,
    decimals = decimals,
)

fun PaymentPrice.toGem(): GemPaymentPrice = GemPaymentPrice(
    symbol = symbol,
    value = value,
    decimals = decimals,
)

fun GemPaymentQuote.toPrimitives(): PaymentQuote = PaymentQuote(
    id = id,
    paymentId = paymentId,
    amount = amount.toPrimitives(),
    expiresAt = expiresAt?.secondsToMillis(),
    collectDataUrl = collectDataUrl,
    providerData = providerData,
)

fun PaymentQuote.toGem(): GemPaymentQuote = GemPaymentQuote(
    id = id,
    paymentId = paymentId,
    amount = amount.toGem(),
    expiresAt = expiresAt?.millisToSeconds(),
    collectDataUrl = collectDataUrl,
    providerData = providerData,
)

fun GemPaymentQuotes.toPrimitives(): PaymentQuotes = PaymentQuotes(
    merchant = merchant.toPrimitives(),
    price = price?.toPrimitives(),
    expiresAt = expiresAt?.secondsToMillis(),
    quotes = quotes.map { it.toPrimitives() },
)

fun PaymentQuotes.toGem(): GemPaymentQuotes = GemPaymentQuotes(
    merchant = merchant.toGem(),
    price = price?.toGem(),
    expiresAt = expiresAt?.millisToSeconds(),
    quotes = quotes.map { it.toGem() },
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

fun GemPaymentOptions.toPrimitives(): PaymentOptions = when (this) {
    is GemPaymentOptions.Quotes -> PaymentOptions.Quotes(v1.toPrimitives())
    is GemPaymentOptions.Outcome -> PaymentOptions.Outcome(v1.toPrimitives())
}
