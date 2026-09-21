package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.model.text
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemSlippageSession
import java.util.Locale

class SwapSlippageTest {

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
    fun suggestions_carryThePercentCoreFormatted() {
        val suggestions = GemSlippageSession(isAuto = false, bps = 100u).viewState().suggestions

        assertEquals(listOf(30u, 50u, 300u), suggestions.map { it.bps })
        assertEquals(listOf("0.3%", "0.5%", "3%"), suggestions.map { it.percent.text(Locale.US) })
        assertEquals(listOf("%0,3", "%0,5", "%3"), suggestions.map { it.percent.text(Locale.forLanguageTag("tr")) })
    }

    @Test
    fun sanitize_limitsDigitsToWhatCoreAllows() {
        val state = GemSlippageSession(isAuto = false, bps = 100u).viewState()

        assertEquals("0.11", SwapSlippage.sanitize("0.111111", state.maximumFractionDigits, state.maximumIntegerDigits))
        assertEquals("33", SwapSlippage.sanitize("33333312312", state.maximumFractionDigits, state.maximumIntegerDigits))
    }
}
