package com.gemwallet.android.testkit

import com.wallet.core.primitives.FiatProvider
import com.wallet.core.primitives.FiatQuote
import com.wallet.core.primitives.FiatQuoteType

fun mockFiatProvider(
    id: String = "mercuryo",
    name: String = "Mercuryo",
    imageUrl: String? = null,
) = FiatProvider(
    id = id,
    name = name,
    imageUrl = imageUrl,
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
    provider = provider,
    type = type,
    fiatAmount = fiatAmount,
    fiatCurrency = fiatCurrency,
    cryptoAmount = cryptoAmount,
    paymentMethods = emptyList(),
)
