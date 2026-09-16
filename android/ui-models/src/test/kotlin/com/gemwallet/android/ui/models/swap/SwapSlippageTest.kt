package com.gemwallet.android.ui.models.swap

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemSlippageSession

class SwapSlippageTest {
    private val slippagePercent = { bps: UInt -> bps.toDouble() / 100 }

    @Test
    fun format_trimsTrailingZeros() {
        assertEquals("1", SwapSlippage.format(100u, slippagePercent))
        assertEquals("0.5", SwapSlippage.format(50u, slippagePercent))
        assertEquals("0.1", SwapSlippage.format(10u, slippagePercent))
        assertEquals("5", SwapSlippage.format(500u, slippagePercent))
    }

    @Test
    fun parseBps_handsTheTypedPercentToCore() {
        val slippageBps = { percent: Double -> (percent * 100).toUInt() }

        assertEquals(50u, SwapSlippage.parseBps("0.5", slippageBps))
        assertEquals(500u, SwapSlippage.parseBps("5", slippageBps))
        assertEquals(2000u, SwapSlippage.parseBps("20", slippageBps))
    }

    @Test
    fun parseBps_returnsNullWithoutANumber() {
        val slippageBps = { percent: Double -> (percent * 100).toUInt() }

        assertNull(SwapSlippage.parseBps("", slippageBps))
        assertNull(SwapSlippage.parseBps("abc", slippageBps))
    }

    @Test
    fun suggestions_formatToExpectedLabels() {
        val suggestions = GemSlippageSession(isAuto = false, bps = 100u).viewState().suggestionsBps

        assertEquals(listOf(30u, 50u, 300u), suggestions)
        assertEquals(listOf("0.3%", "0.5%", "3%"), suggestions.map { SwapSlippage.percentLabel(it, slippagePercent) })
    }

    @Test
    fun sanitize_limitsDigitsToWhatCoreAllows() {
        val state = GemSlippageSession(isAuto = false, bps = 100u).viewState()

        assertEquals("0.11", SwapSlippage.sanitize("0.111111", state))
        assertEquals("33", SwapSlippage.sanitize("33333312312", state))
    }
}
