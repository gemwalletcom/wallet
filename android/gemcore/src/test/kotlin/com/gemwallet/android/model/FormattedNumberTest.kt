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
            exact = null,
        )

        assertEquals("5.2 ATOM", amount.text(Locale.US))
        assertEquals("5.21 ATOM", amount.copy(rounding = GemNumberRounding.TO_NEAREST).text(Locale.US))
    }

    @Test
    fun `each value formats on its own even when the same currency formatted a signed rounded value before`() {
        val fiat = GemFormattedNumber(
            value = 1234.567,
            unit = GemNumberUnit.Currency(code = "USD"),
            display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 2u, max = 2u)),
            notation = GemNumberNotation.SIGNED,
            tone = GemValueTone.PLAIN,
            rounding = GemNumberRounding.TOWARD_ZERO,
            exact = null,
        )

        assertEquals("+$1,234.56", fiat.text(Locale.US))
        assertEquals("$1,234.57", fiat.copy(notation = GemNumberNotation.PLAIN, rounding = GemNumberRounding.TO_NEAREST).text(Locale.US))
        assertEquals("€1,234.57", fiat.copy(unit = GemNumberUnit.Currency(code = "EUR"), notation = GemNumberNotation.PLAIN, rounding = GemNumberRounding.TO_NEAREST).text(Locale.US))
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
            exact = null,
        )
        val turkish = Locale.forLanguageTag("tr")

        assertEquals("%5,23", percent.text(turkish))
        assertEquals("+%5,23", percent.copy(notation = GemNumberNotation.SIGNED).text(turkish))
        assertEquals("%5,23", percent.copy(value = -5.23).text(turkish))
        assertEquals("5.23%", percent.text(Locale.US))
    }

    @Test
    fun `a full amount reads its exact digits`() {
        val spent = GemFormattedNumber(
            value = -123.456789012345678901,
            unit = GemNumberUnit.Symbol(symbol = "ETH"),
            display = GemNumberDisplay.Number(precision = GemPrecision.Fraction(min = 0u, max = 32u)),
            notation = GemNumberNotation.SIGNED,
            tone = GemValueTone.NEGATIVE,
            rounding = GemNumberRounding.TOWARD_ZERO,
            exact = "123.456789012345678901",
        )

        assertEquals("every digit past what a double holds", "-123.456789012345678901 ETH", spent.text(Locale.US))
        assertEquals("the digits follow the reader's separator", "0,5 SOL", spent.copy(value = 0.5, unit = GemNumberUnit.Symbol(symbol = "SOL"), notation = GemNumberNotation.PLAIN, exact = "0.5").text(Locale.GERMANY))
    }
}
