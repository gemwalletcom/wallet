package com.gemwallet.android.domains.price

import org.junit.Assert.assertEquals
import org.junit.Test

class PriceChangeTest {

    @Test
    fun percentage_positiveMovement() {
        assertEquals(10.0, PriceChange.percentage(from = 100.0, to = 110.0), 0.0001)
    }

    @Test
    fun percentage_negativeMovement() {
        assertEquals(-25.0, PriceChange.percentage(from = 200.0, to = 150.0), 0.0001)
    }

    @Test
    fun percentage_zeroFromReturnsZero() {
        assertEquals(0.0, PriceChange.percentage(from = 0.0, to = 50.0), 0.0001)
    }

    @Test
    fun percentage_equalValuesReturnZero() {
        assertEquals(0.0, PriceChange.percentage(from = 100.0, to = 100.0), 0.0001)
    }

    @Test
    fun percentage_doublingReturnsHundredPercent() {
        assertEquals(100.0, PriceChange.percentage(from = 50.0, to = 100.0), 0.0001)
    }

    @Test
    fun amount_positivePercentageOfTotal() {
        assertEquals(10.0, PriceChange.amount(percentage = 10.0, value = 110.0), 0.0001)
    }

    @Test
    fun amount_negativePercentageOfTotal() {
        assertEquals(-10.0, PriceChange.amount(percentage = -10.0, value = 90.0), 0.0001)
    }

    @Test
    fun amount_zeroPercentageReturnsZero() {
        assertEquals(0.0, PriceChange.amount(percentage = 0.0, value = 100.0), 0.0001)
    }
}
