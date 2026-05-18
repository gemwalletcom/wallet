package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigDecimal
import java.math.BigInteger

class CryptoFiatConverterTest {

    private val converter = CryptoFiatConverter()

    @Test
    fun toFiat() {
        assertEquals(
            0,
            converter.toFiat(Crypto(BigInteger("100000000")), decimals = 8, price = 50_000.0)
                .atomicValue.compareTo(BigDecimal("50000"))
        )
        assertEquals(
            0,
            converter.toFiat(Crypto(BigInteger.ZERO), decimals = 8, price = 50_000.0)
                .atomicValue.compareTo(BigDecimal.ZERO)
        )
    }

    @Test
    fun toCrypto() {
        assertEquals(
            BigInteger("100000000"),
            converter.toCrypto(Fiat(BigDecimal("50000")), decimals = 8, price = 50_000.0).atomicValue
        )
        assertEquals(
            BigInteger.ZERO,
            converter.toCrypto(Fiat(BigDecimal("100")), decimals = 8, price = 0.0).atomicValue
        )
    }
}
