package com.gemwallet.android.ext

import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentProviderName
import com.wallet.core.primitives.PaymentRequest
import uniffi.gemstone.GemPayment
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentProviderName
import uniffi.gemstone.GemPaymentRequest

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

fun GemPaymentProviderName.toPrimitives(): PaymentProviderName = when (this) {
    GemPaymentProviderName.SOLANA_PAY -> PaymentProviderName.SolanaPay
    GemPaymentProviderName.WALLET_CONNECT_PAY -> PaymentProviderName.WalletConnectPay
}

fun PaymentProviderName.toGem(): GemPaymentProviderName = when (this) {
    PaymentProviderName.SolanaPay -> GemPaymentProviderName.SOLANA_PAY
    PaymentProviderName.WalletConnectPay -> GemPaymentProviderName.WALLET_CONNECT_PAY
}
