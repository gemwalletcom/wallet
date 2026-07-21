package com.gemwallet.android.model

import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.math.BigDecimal
import java.math.BigInteger
import java.util.Locale

class CryptoFiatConverterTest {

    private val defaultLocale = Locale.getDefault()

    @After
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun toFiat() {
        assertEquals(
            0,
            CryptoFiatConverter.toFiat(Crypto(BigInteger("100000000")), decimals = 8, price = 50_000.0)
                .atomicValue.compareTo(BigDecimal("50000"))
        )
    }

    @Test
    fun toCrypto() {
        assertEquals(
            BigInteger("100000000"),
            CryptoFiatConverter.toCrypto(Fiat(BigDecimal("50000")), decimals = 8, price = 50_000.0)?.atomicValue
        )
    }

    @Test
    fun toCryptoInvalidPriceReturnsNull() {
        assertNull(CryptoFiatConverter.toCrypto(Fiat(BigDecimal("100")), decimals = 8, price = 0.0))
        assertNull(CryptoFiatConverter.toCrypto(Fiat(BigDecimal("100")), decimals = 8, price = -1.0))
        assertNull(CryptoFiatConverter.toCryptoAtDisplayPrecision(Fiat(BigDecimal("1")), decimals = 18, price = 0.0))
    }

    @Test
    fun toCryptoAtDisplayPrecision() {
        for (locale in listOf(Locale.US, Locale.GERMANY, Locale.FRANCE)) {
            Locale.setDefault(locale)
            assertEquals(BigInteger.valueOf(1302L), atDisplayPrecision("1", 76_800.0, 8))
            assertEquals(BigInteger.valueOf(40_000_000L), atDisplayPrecision("1", 2.5, 8))
            assertEquals(BigInteger.valueOf(1_250_000L), atDisplayPrecision("1", 80.0, 8))
            assertEquals(BigInteger.valueOf(12_200L), atDisplayPrecision("1", 8192.0, 8))
            assertEquals(BigInteger.valueOf(400_000_000L), atDisplayPrecision("10", 2.5, 8))
            assertEquals(BigInteger.ZERO, atDisplayPrecision("0", 2.5, 8))
        }
    }

    @Test
    fun toCryptoAtDisplayPrecisionAboveGroupingThreshold() {
        for (locale in listOf(Locale.US, Locale.GERMANY, Locale.FRANCE)) {
            Locale.setDefault(locale)
            assertEquals(BigInteger.valueOf(1_000_000_000L), atDisplayPrecision("1000", 1.0, 6))
            assertEquals(BigInteger.valueOf(2_000_500_000L), atDisplayPrecision("2000.5", 1.0, 6))
            assertEquals(BigInteger.valueOf(1_234_560_000L), atDisplayPrecision("1234.56", 1.0, 6))
            assertEquals(BigInteger.valueOf(500_000_000_000L), atDisplayPrecision("1000000", 2.0, 6))
            assertEquals(BigInteger.valueOf(40_000_000_000L), atDisplayPrecision("100000", 2.5, 6))
        }
    }

    private fun atDisplayPrecision(fiat: String, price: Double, decimals: Int): BigInteger? =
        CryptoFiatConverter.toCryptoAtDisplayPrecision(Fiat(BigDecimal(fiat)), decimals, price)?.atomicValue
}
