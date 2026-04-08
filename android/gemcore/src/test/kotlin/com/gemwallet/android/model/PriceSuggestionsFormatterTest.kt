package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class PriceSuggestionsFormatterTest {

    @Test
    fun testHighPrice() {
        val result = PriceSuggestionsFormatter.roundedValues(95_432.0)
        assertEquals(2, result.size)
        assertEquals(90_000.0, result[0], 0.01)
        assertEquals(100_000.0, result[1], 0.01)
    }

    @Test
    fun testMediumPrice() {
        val result = PriceSuggestionsFormatter.roundedValues(767.55)
        assertEquals(2, result.size)
        assertEquals(700.0, result[0], 0.01)
        assertEquals(800.0, result[1], 0.01)
    }

    @Test
    fun testLowPrice() {
        val result = PriceSuggestionsFormatter.roundedValues(0.2829)
        assertEquals(2, result.size)
        assertEquals(0.26, result[0], 0.01)
        assertEquals(0.30, result[1], 0.01)
    }

    @Test
    fun testVeryLowPrice() {
        val result = PriceSuggestionsFormatter.roundedValues(0.005)
        assertTrue(result.isEmpty())
    }

    @Test
    fun testZeroPrice() {
        val result = PriceSuggestionsFormatter.roundedValues(0.0)
        assertTrue(result.isEmpty())
    }

    @Test
    fun testNegativePercent() {
        val result = PriceSuggestionsFormatter.roundedValues(100.0, -5.0)
        assertTrue(result.isEmpty())
    }
}
