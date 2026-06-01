package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.util.Locale

class NumericFormatterTest {

    private val us = NumericFormatter(locale = Locale.US)
    private val de = NumericFormatter(locale = Locale.GERMANY)

    @Test
    fun decimal() {
        assertEquals("0.00", us.string(0.0))
        assertEquals("11.12", us.string(11.12))
        assertEquals("11.00", us.string(11.0))
        assertEquals("12,000,123.00", us.string(12_000_123.0))
        assertEquals("-1.20", us.string(-1.2))
        assertEquals("0.12", us.string(0.12))
        assertEquals("0.00012", us.string(0.00012))
        assertEquals("0.0000000002", us.string(0.0000000002))
        assertEquals("0.00", us.string(0.0000000000001))
        assertEquals("", us.string(Double.NaN))
        assertEquals("", us.string(Double.POSITIVE_INFINITY))
    }

    @Test
    fun withSymbol() {
        assertEquals("1,234.56 BTC", us.string(1234.56, "BTC"))
        assertEquals("0.0001234 BTC", us.string(0.0001234, "BTC"))
        assertEquals("0.00 BTC", us.string(0.0, "BTC"))
    }

    @Test
    fun locale() {
        assertEquals("29,73", de.string(29.73))
        assertEquals("12.000.123,00", de.string(12_000_123.0))
    }

    @Test
    fun parseDouble() {
        assertEquals(11.12, us.double("11.12")!!, 1e-9)
        assertEquals(11.0, us.double("11")!!, 1e-9)
        assertNull(us.double(""))
        assertNull(us.double("   "))
        assertNull(us.double("abc"))
        assertEquals(29.73, de.double("29,73")!!, 1e-9)
        assertEquals(12_000.5, us.double("12,000.5")!!, 1e-9)
        assertEquals(12_000.5, de.double("12.000,5")!!, 1e-9)
        assertEquals(12.0, us.double("12abc")!!, 1e-9)
    }
}
