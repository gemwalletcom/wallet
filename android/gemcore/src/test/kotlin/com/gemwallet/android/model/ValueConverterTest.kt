package com.gemwallet.android.model

import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger
import java.util.Locale

class ValueConverterTest {

    private val defaultLocale = Locale.getDefault()

    @After
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun convertToAmount() {
        for (locale in listOf(Locale.US, Locale.GERMANY, Locale.FRANCE)) {
            Locale.setDefault(locale)
            assertEquals(BigInteger.valueOf(1302L), ValueConverter.convertToAmount("1", 76_800.0, 8).atomicValue)
            assertEquals(BigInteger.valueOf(40_000_000L), ValueConverter.convertToAmount("1", 2.5, 8).atomicValue)
            assertEquals(BigInteger.valueOf(1_250_000L), ValueConverter.convertToAmount("1", 80.0, 8).atomicValue)
            assertEquals(BigInteger.valueOf(12_200L), ValueConverter.convertToAmount("1", 8192.0, 8).atomicValue)
            assertEquals(BigInteger.valueOf(400_000_000L), ValueConverter.convertToAmount("10", 2.5, 8).atomicValue)
            assertEquals(BigInteger.ZERO, ValueConverter.convertToAmount("1", 0.0, 18).atomicValue)
            assertEquals(BigInteger.ZERO, ValueConverter.convertToAmount("0", 2.5, 8).atomicValue)
        }
    }

    @Test
    fun convertToFiat() {
        assertEquals("2.5", ValueConverter.convertToFiat("1", 2.5).toPlainString())
        assertEquals("25.0", ValueConverter.convertToFiat("10", 2.5).toPlainString())
        assertEquals("0.0", ValueConverter.convertToFiat("0", 2.5).toPlainString())
    }
}
