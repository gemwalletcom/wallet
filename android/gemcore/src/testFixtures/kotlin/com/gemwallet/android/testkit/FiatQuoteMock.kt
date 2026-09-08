package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
import java.math.BigInteger
import uniffi.gemstone.FiatProvider
import uniffi.gemstone.FiatQuote
import uniffi.gemstone.GemFiatQuoteRow

fun mockFiatProvider(
    id: FiatProviderName = FiatProviderName.Mercuryo,
    name: String = "Mercuryo",
    imageUrl: String? = null,
) = FiatProvider(
    id = id.toGem(),
    name = name,
    imageUrl = imageUrl,
    priority = null,
    thresholdBps = null,
    enabled = true,
    buyEnabled = true,
    sellEnabled = true,
    paymentMethods = emptyList(),
)

fun mockFiatQuote(
    id: String = "quote-1",
    provider: FiatProvider = mockFiatProvider(),
    type: FiatQuoteType = FiatQuoteType.Buy,
    fiatAmount: Double = 100.0,
    fiatCurrency: String = "USD",
    cryptoAmount: Double = 0.17,
) = FiatQuote(
    id = id,
    asset = mockAsset().toGem(),
    provider = provider,
    quoteType = type.toGem(),
    fiatAmount = fiatAmount,
    fiatCurrency = fiatCurrency,
    cryptoAmount = cryptoAmount,
    value = BigInteger.ZERO,
    latency = 0uL,
    paymentMethods = emptyList(),
)

fun mockFiatQuoteRow(
    quoteId: String = "quote-1",
    provider: FiatProviderName = FiatProviderName.Mercuryo,
    providerName: String = "Mercuryo",
    providerImageUrl: String? = null,
    cryptoAmount: Double = 0.17,
    fiatAmount: Double = 100.0,
    rate: Double? = null,
) = GemFiatQuoteRow(
    quoteId = quoteId,
    provider = provider.toGem(),
    providerName = providerName,
    providerImageUrl = providerImageUrl,
    cryptoAmount = cryptoAmount,
    fiatAmount = fiatAmount,
    rate = rate,
)
