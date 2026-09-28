package com.gemwallet.android.model

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.formattedCurrency
import java.util.Locale

class CurrencyTextTest {

    private fun currency(style: GemCurrencyStyle, code: String, locale: Locale): (Double) -> String = { formattedCurrency(it, code, style).text(locale) }

    private val currencyUS = currency(GemCurrencyStyle.CURRENCY, "USD", Locale.US)
    private val currencyUK = currency(GemCurrencyStyle.CURRENCY, "GBP", Locale.UK)
    private val fiatUS = currency(GemCurrencyStyle.FIAT, "USD", Locale.US)
    private val shortUS = currency(GemCurrencyStyle.SHORT, "USD", Locale.US)
    private val shortUK = currency(GemCurrencyStyle.SHORT, "GBP", Locale.UK)

    @Test
    fun short_readsDustBelowTheAmountThreshold() {
        assertEquals("<$0.0001", shortUS(0.00000783))
        assertEquals("<£0.0001", shortUK(0.00000783))
        assertEquals("$0.0001", shortUS(0.0001))
        assertEquals("$0.00", shortUS(0.0))
        assertEquals("$0.0345", shortUS(0.0345))
        assertEquals("$1,234.50", shortUS(1234.5))
        assertEquals("$0.00000783", currencyUS(0.00000783))
    }

    @Test
    fun currency_normal() {
        assertEquals("$0.00", currencyUS(0.0))
        assertEquals("$11.12", currencyUS(11.12))
        assertEquals("$11.00", currencyUS(11.0))
        assertEquals("$12,000,123.00", currencyUS(12_000_123.0))
        assertEquals("-$1.20", currencyUS(-1.2))
    }

    @Test
    fun currency_smallValueAdaptive() {
        assertEquals("$0.99", currencyUS(0.99))
        assertEquals("$1.90", currencyUS(1.89999))
        assertEquals("$0.0345", currencyUS(0.0345))
        assertEquals("$0.0001235", currencyUS(0.000123456))
        assertEquals("$0.00000123", currencyUS(0.00000123))
        assertEquals("$0.0000000002", currencyUS(0.0000000002))
        assertEquals("$0.00", currencyUS(0.0000000000001))
    }

    @Test
    fun currency_gbpLocale() {
        assertEquals("£0.0002", currencyUK(0.0002))
        assertEquals("£11.12", currencyUK(11.12))
        assertEquals("£12,000,123.00", currencyUK(12_000_123.0))
    }

    @Test
    fun fiat_alwaysTwoPlaces() {
        assertEquals("$0.50", fiatUS(0.5))
        assertEquals("$0.00", fiatUS(0.0001234))
        assertEquals("$1,234.56", fiatUS(1234.56))
        assertEquals("$11.00", fiatUS(11.0))
    }
}
