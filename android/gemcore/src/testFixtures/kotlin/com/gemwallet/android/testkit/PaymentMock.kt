package com.gemwallet.android.testkit

import com.gemwallet.android.model.PaymentData
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentMerchant
import com.wallet.core.primitives.PaymentQuote
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentMerchant
import uniffi.gemstone.GemPaymentQuote
import uniffi.gemstone.GemPaymentQuotes

fun mockGemPaymentQuote(
    id: String = "option_1",
    link: GemPaymentLink = GemPaymentLink.WalletConnectPay("pay_1"),
    assetId: String = "ethereum",
    value: String = "14192816625800",
    collectDataUrl: String? = null,
    providerData: String = "{}",
) = GemPaymentQuote(
    id = id,
    link = link,
    assetId = assetId,
    value = value,
    collectDataUrl = collectDataUrl,
    providerData = providerData,
)

fun mockGemPaymentMerchant(
    name: String = "Merchant",
    iconUrl: String? = null,
) = GemPaymentMerchant(
    name = name,
    iconUrl = iconUrl,
)

fun mockGemPaymentQuotes(
    merchant: GemPaymentMerchant = mockGemPaymentMerchant(),
    quotes: List<GemPaymentQuote> = listOf(mockGemPaymentQuote()),
) = GemPaymentQuotes(
    merchant = merchant,
    price = null,
    quotes = quotes,
)

fun mockPaymentQuote(
    id: String = "option_1",
    link: PaymentLink = PaymentLink.WalletConnectPay("pay_1"),
    assetId: AssetId = AssetId(Chain.Ethereum),
    value: String = "1",
    collectDataUrl: String? = null,
    providerData: String = "{}",
) = PaymentQuote(
    id = id,
    link = link,
    assetId = assetId,
    value = value,
    collectDataUrl = collectDataUrl,
    providerData = providerData,
)

fun mockPaymentData(
    quote: PaymentQuote = mockPaymentQuote(),
    merchant: PaymentMerchant = PaymentMerchant(name = "Merchant", iconUrl = null),
) = PaymentData(
    quote = quote,
    merchant = merchant,
)
