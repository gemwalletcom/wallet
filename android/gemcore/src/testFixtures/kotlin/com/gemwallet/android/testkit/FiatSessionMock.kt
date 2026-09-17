package com.gemwallet.android.testkit

import uniffi.gemstone.FiatQuoteType
import uniffi.gemstone.GemFiatOperation
import uniffi.gemstone.GemFiatQuotePhase
import uniffi.gemstone.GemFiatSession
import java.math.BigInteger

fun mockGemFiatSession(
    quoteType: FiatQuoteType = FiatQuoteType.BUY,
    amount: UInt? = null,
): GemFiatSession {
    val operation = { type: FiatQuoteType, default: UInt ->
        val value = amount?.takeIf { type == quoteType } ?: default
        GemFiatOperation(type, value.toString(), emptyList(), null, GemFiatQuotePhase.Loading(value.toDouble()))
    }
    return GemFiatSession(
        quoteType = quoteType,
        buy = operation(FiatQuoteType.BUY, 50u),
        sell = operation(FiatQuoteType.SELL, 100u),
        available = BigInteger.ZERO,
    )
}
