package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.model.text
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemNumberFormat
import uniffi.gemstone.GemSlippageSelection
import uniffi.gemstone.newSlippageSession
import java.util.Locale

class SwapSlippageTest {

    @Test
    fun suggestions_carryThePercentCoreFormatted() {
        val suggestions = newSlippageSession(GemSlippageSelection.Auto, "ethereum", GemNumberFormat(",")).viewState().suggestions

        assertEquals(listOf(30u, 50u, 300u), suggestions.map { it.bps })
        assertEquals(listOf("0,3", "0,5", "3"), suggestions.map { it.input })
        assertEquals(listOf("0.3%", "0.5%", "3%"), suggestions.map { it.percent.text(Locale.US) })
        assertEquals(listOf("%0,3", "%0,5", "%3"), suggestions.map { it.percent.text(Locale.forLanguageTag("tr")) })
    }
}
