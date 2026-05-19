package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.math.BigInteger
import java.util.Locale

class ValueConverterTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    private val converter = ValueConverter(
        formatter = ValueFormatter(style = ValueFormatter.Style.Auto, locale = Locale.US),
    )

    @Test
    fun convertToAmount() {
        assertEquals(BigInteger.valueOf(1302L), converter.convertToAmount("1", 76_800.0, 8).atomicValue)
        assertEquals(BigInteger.valueOf(40_000_000L), converter.convertToAmount("1", 2.5, 8).atomicValue)
        assertEquals(BigInteger.valueOf(1_250_000L), converter.convertToAmount("1", 80.0, 8).atomicValue)
        assertEquals(BigInteger.valueOf(12_200L), converter.convertToAmount("1", 8192.0, 8).atomicValue)
        assertEquals(BigInteger.valueOf(400_000_000L), converter.convertToAmount("10", 2.5, 8).atomicValue)
        assertEquals(BigInteger.ZERO, converter.convertToAmount("1", 0.0, 18).atomicValue)
        assertEquals(BigInteger.ZERO, converter.convertToAmount("0", 2.5, 8).atomicValue)
    }

    @Test
    fun convertToFiat() {
        assertEquals("2.5", converter.convertToFiat("1", 2.5).toPlainString())
        assertEquals("25.0", converter.convertToFiat("10", 2.5).toPlainString())
        assertEquals("0.0", converter.convertToFiat("0", 2.5).toPlainString())
    }
}
