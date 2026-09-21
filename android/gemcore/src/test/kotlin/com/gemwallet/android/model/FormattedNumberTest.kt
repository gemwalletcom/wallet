package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberRounding
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemValueTone
import java.util.Locale

class FormattedNumberTest {

    @Test
    fun `an amount never reads above what is held`() {
        val amount = GemFormattedNumber(
            value = 5.205516,
            unit = GemNumberUnit.Symbol(symbol = "ATOM"),
            display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 0u, max = 2u)),
            notation = GemNumberNotation.PLAIN,
            tone = GemValueTone.PLAIN,
            rounding = GemNumberRounding.TOWARD_ZERO,
        )

        assertEquals("5.2 ATOM", amount.text(Locale.US))
        assertEquals("5.21 ATOM", amount.copy(rounding = GemNumberRounding.TO_NEAREST).text(Locale.US))
    }

    @Test
    fun `a percent keeps the sign the locale puts in front of it`() {
        val percent = GemFormattedNumber(
            value = 5.23,
            unit = GemNumberUnit.Percent,
            display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 0u, max = 2u)),
            notation = GemNumberNotation.PLAIN,
            tone = GemValueTone.PLAIN,
            rounding = GemNumberRounding.TO_NEAREST,
        )
        val turkish = Locale.forLanguageTag("tr")

        assertEquals("%5,23", percent.text(turkish))
        assertEquals("+%5,23", percent.copy(notation = GemNumberNotation.SIGNED).text(turkish))
        assertEquals("%5,23", percent.copy(value = -5.23).text(turkish))
        assertEquals("5.23%", percent.text(Locale.US))
    }
}
