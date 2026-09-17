package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemBigUint
import uniffi.gemstone.PaymentInvoice
import uniffi.gemstone.PaymentLink
import uniffi.gemstone.PaymentMerchant
import uniffi.gemstone.PaymentQuote

fun mockPaymentInvoice(
    link: PaymentLink = PaymentLink.SolanaPay("https://example.com/pay"),
    quotes: List<Asset> = emptyList(),
) = PaymentInvoice(
    link = link,
    merchant = PaymentMerchant(name = "Merchant", icon = "https://example.com/icon.png"),
    price = null,
    quotes = quotes.map { PaymentQuote(id = it.id.toIdentifier(), assetId = it.id.toIdentifier(), value = GemBigUint("1")) },
    verification = null,
)
