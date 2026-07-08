package com.gemwallet.android.ui.models.swap

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SwapSlippageTest {
    @Test
    fun format_trimsTrailingZeros() {
        assertEquals("1", SwapSlippage.format(100u))
        assertEquals("0.5", SwapSlippage.format(50u))
        assertEquals("0.1", SwapSlippage.format(10u))
        assertEquals("5", SwapSlippage.format(500u))
    }

    @Test
    fun parseBps_convertsPercentToBps() {
        assertEquals(50u, SwapSlippage.parseBps("0.5"))
        assertEquals(500u, SwapSlippage.parseBps("5"))
        assertEquals(2000u, SwapSlippage.parseBps("20"))
    }

    @Test
    fun parseBps_returnsNullForEmptyOrZero() {
        assertNull(SwapSlippage.parseBps(""))
        assertNull(SwapSlippage.parseBps("0"))
    }

    @Test
    fun parseBps_clampsToMaximum() {
        assertEquals(2000u, SwapSlippage.parseBps("25"))
    }

    @Test
    fun isOverMax_detectsAboveMaximum() {
        assertTrue(SwapSlippage.isOverMax("25"))
        assertFalse(SwapSlippage.isOverMax("20"))
        assertFalse(SwapSlippage.isOverMax(""))
    }

    @Test
    fun sanitize_limitsFractionAndIntegerDigits() {
        assertEquals("0.11", SwapSlippage.sanitize("0.111111"))
        assertEquals("33", SwapSlippage.sanitize("33333312312"))
    }
}
