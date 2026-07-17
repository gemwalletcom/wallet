package com.gemwallet.android.ui.models

import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class AmountInputTypeTest {

    @Test
    fun cryptoKeepsTypedPrecisionWhileFiatRoundsToDisplayPrecision() {
        assertEquals(
            BigInteger("1000123456"),
            AmountInputType.Crypto.getAmount("1000.123456", decimals = 6, price = 1.0).atomicValue,
        )
        assertEquals(
            BigInteger("1000120000"),
            AmountInputType.Fiat.getAmount("1000.123456", decimals = 6, price = 1.0).atomicValue,
        )
    }

    @Test
    fun fiatKeepsWholeAmountsAboveGroupingThreshold() {
        assertEquals(BigInteger("1000000000"), AmountInputType.Fiat.getAmount("1000", decimals = 6, price = 1.0).atomicValue)
        assertEquals(BigInteger("400000000"), AmountInputType.Fiat.getAmount("10", decimals = 8, price = 2.5).atomicValue)
    }
}
