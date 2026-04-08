package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Test

class PercentageSuggestionsFormatterTest {

    @Test
    fun testLowPrice() {
        assertEquals(listOf(5, 10, 15), PercentageSuggestionsFormatter.suggestions(50.0))
    }

    @Test
    fun testMediumPrice() {
        assertEquals(listOf(3, 6, 9), PercentageSuggestionsFormatter.suggestions(500.0))
    }

    @Test
    fun testHighPrice() {
        assertEquals(listOf(2, 4, 6), PercentageSuggestionsFormatter.suggestions(10_000.0))
    }

    @Test
    fun testBoundaryAt100() {
        assertEquals(listOf(3, 6, 9), PercentageSuggestionsFormatter.suggestions(100.0))
    }

    @Test
    fun testBoundaryAt10000() {
        assertEquals(listOf(2, 4, 6), PercentageSuggestionsFormatter.suggestions(10_000.0))
    }

    @Test
    fun testVeryLowPrice() {
        assertEquals(listOf(5, 10, 15), PercentageSuggestionsFormatter.suggestions(0.01))
    }
}
