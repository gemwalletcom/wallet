package com.gemwallet.android.ui.models.swap

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SwapSlippageUIModelTest {
    private val model = SwapSlippageUIModel(
        defaultBps = 100u,
        suggestionsBps = listOf(30u, 50u, 300u),
        minBps = 10u,
        maxBps = 2000u,
        highWarningBps = 300u,
    )

    @Test
    fun format_trimsTrailingZeros() {
        assertEquals("1", SwapSlippageUIModel.format(100u))
        assertEquals("0.5", SwapSlippageUIModel.format(50u))
        assertEquals("0.1", SwapSlippageUIModel.format(10u))
        assertEquals("5", SwapSlippageUIModel.format(500u))
    }

    @Test
    fun parseBps_convertsPercentToBps() {
        assertEquals(50u, model.parseBps("0.5"))
        assertEquals(500u, model.parseBps("5"))
        assertEquals(2000u, model.parseBps("20"))
    }

    @Test
    fun parseBps_returnsNullForEmptyOrZero() {
        assertNull(model.parseBps(""))
        assertNull(model.parseBps("0"))
    }

    @Test
    fun parseBps_clampsToMaximum() {
        assertEquals(2000u, model.parseBps("25"))
    }

    @Test
    fun isOverMax_detectsAboveMaximum() {
        assertTrue(model.isOverMax("25"))
        assertFalse(model.isOverMax("20"))
        assertFalse(model.isOverMax(""))
    }

    @Test
    fun isBelowMin_detectsBelowMinimum() {
        assertTrue(model.isBelowMin("0.05"))
        assertFalse(model.isBelowMin("0.1"))
        assertFalse(model.isBelowMin("5"))
        assertFalse(model.isBelowMin("0"))
        assertFalse(model.isBelowMin(""))
    }

    @Test
    fun boundLabels_formatFromBps() {
        assertEquals("0.1%", model.minPercentLabel)
        assertEquals("20%", model.maxPercentLabel)
    }

    @Test
    fun sanitize_limitsFractionAndIntegerDigits() {
        assertEquals("0.11", model.sanitize("0.111111"))
        assertEquals("33", model.sanitize("33333312312"))
    }
}
