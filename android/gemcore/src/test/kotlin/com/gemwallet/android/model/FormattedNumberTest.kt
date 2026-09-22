package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberRounding
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.formattedCurrency
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

    @Test
    fun `a short price reads like the currency formatter`() {
        val formatter = CurrencyFormatter(style = GemCurrencyStyle.SHORT, currency = Currency.USD, locale = Locale.US)

        listOf(0.00000783, 0.0001, 0.0345, 1234.5).forEach { value ->
            assertEquals(formatter.string(value), formattedCurrency(value, "USD", GemCurrencyStyle.SHORT).text(Locale.US))
        }
        assertEquals("<$0.0001", formattedCurrency(0.00000783, "USD", GemCurrencyStyle.SHORT).text(Locale.US))
    }
}
