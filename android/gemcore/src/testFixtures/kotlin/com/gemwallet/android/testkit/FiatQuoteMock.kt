package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
import java.math.BigInteger
import uniffi.gemstone.FiatProvider
import uniffi.gemstone.FiatQuote
import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemFiatQuoteRow
import uniffi.gemstone.formattedCurrency
import uniffi.gemstone.formattedAmount
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.GemCurrencyStyle

fun mockFiatQuote(quoteType: FiatQuoteType = FiatQuoteType.Buy) = FiatQuote(
    id = "quote-1",
    asset = mockAsset().toGem(),
    provider = FiatProvider(
        id = FiatProviderName.Mercuryo.toGem(),
        name = "Mercuryo",
        imageUrl = null,
        priority = null,
        thresholdBps = null,
        enabled = true,
        buyEnabled = true,
        sellEnabled = true,
        paymentMethods = emptyList(),
    ),
    quoteType = quoteType.toGem(),
    fiatAmount = 100.0,
    fiatCurrency = "USD",
    cryptoAmount = 0.17,
    value = BigInteger.ZERO,
    latency = 0uL,
    paymentMethods = emptyList(),
)

fun mockFiatQuoteRow(
    cryptoAmount: Double = 0.17,
    fiatAmount: Double = 100.0,
    rate: GemAssetRate? = null,
) = GemFiatQuoteRow(
    quoteId = "quote-1",
    provider = FiatProviderName.Mercuryo.toGem(),
    providerName = "Mercuryo",
    providerImageUrl = null,
    cryptoAmount = formattedAmount(cryptoAmount, "BTC", GemValueStyle.AUTO),
    fiatAmount = formattedCurrency(fiatAmount, "USD", GemCurrencyStyle.FIAT),
    rate = rate,
)
