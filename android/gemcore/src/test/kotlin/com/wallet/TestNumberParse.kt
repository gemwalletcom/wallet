package com.wallet

import com.gemwallet.android.math.parseInputNumber
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigDecimal

class TestNumberParse {

    @Test
    fun testUSInput() {
        assertEquals("0.1", "0.1".parseInputNumber().toString())
        assertEquals(BigDecimal("0.1"), "0.1".parseInputNumber())

        assertEquals("0.2", "0.2".parseInputNumber().toString())
        assertEquals(BigDecimal("0.2"), "0.2".parseInputNumber())


        assertEquals("1", "1".parseInputNumber().toString())
        assertEquals(BigDecimal("1"), "1".parseInputNumber())

        assertEquals("1.2", "1.2".parseInputNumber().toString())
        assertEquals(BigDecimal("1.2"), "1.2".parseInputNumber())
        assertEquals(1.2f, "1.2".parseInputNumber().toFloat())

        assertEquals("1.13", "1.13".parseInputNumber().toString())
        assertEquals(BigDecimal("1.13"), "1.13".parseInputNumber())

        assertEquals("0.1234567", "0.1234567".parseInputNumber().toString())
        assertEquals("0.1234567", "0,1234567".parseInputNumber().toString())
        assertEquals("730.1234567", "730.1234567".parseInputNumber().toString())
        assertEquals("730.1234567", "730,1234567".parseInputNumber().toString())
        assertEquals("122726.1234567", "122,726.1234567".parseInputNumber().toString())
    }

    @Test
    fun testTrailingGroupWithoutFractionIsDecimalSeparator() {
        assertEquals(BigDecimal("1.5"), "1,5".parseInputNumber())
        assertEquals(BigDecimal("122.726"), "122,726".parseInputNumber())
        assertEquals(BigDecimal("1.000"), "1,000".parseInputNumber())
        assertEquals(BigDecimal("1.000"), "1.000".parseInputNumber())
    }

    @Test
    fun testRU_UAInput() {
        assertEquals(BigDecimal("0.1234567"), "0.1234567".parseInputNumber())
        assertEquals(BigDecimal("0.1234567"), "0,1234567".parseInputNumber())
        assertEquals(BigDecimal("730.1234567"), "730.1234567".parseInputNumber())
        assertEquals(BigDecimal("730.1234567"), "730,1234567".parseInputNumber())
        assertEquals(BigDecimal("122726.1234567"), "122 726.1234567".parseInputNumber())
    }
}