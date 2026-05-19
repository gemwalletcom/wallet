package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigDecimal
import java.util.Locale

class CurrencyFormatterTest {

    private val currencyUS = CurrencyFormatter(currency = Currency.USD, locale = Locale.US)
    private val currencyUK = CurrencyFormatter(currency = Currency.GBP, locale = Locale.UK)
    private val fiatUS = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD, locale = Locale.US)

    @Test
    fun currency_normal() {
        assertEquals("$0.00", currencyUS.string(0.0))
        assertEquals("$11.12", currencyUS.string(11.12))
        assertEquals("$11.00", currencyUS.string(11.0))
        assertEquals("$12,000,123.00", currencyUS.string(12_000_123.0))
        assertEquals("-$1.20", currencyUS.string(-1.2))
    }

    @Test
    fun currency_smallValueAdaptive() {
        assertEquals("$0.99", currencyUS.string(0.99))
        assertEquals("$1.90", currencyUS.string(1.89999))
        assertEquals("$0.0345", currencyUS.string(0.0345))
        assertEquals("$0.0001235", currencyUS.string(0.000123456))
        assertEquals("$0.00000123", currencyUS.string(0.00000123))
        assertEquals("$0.0000000002", currencyUS.string(0.0000000002))
        assertEquals("$0.00", currencyUS.string(0.0000000000001))
    }

    @Test
    fun currency_gbpLocale() {
        assertEquals("£0.0002", currencyUK.string(0.0002))
        assertEquals("£11.12", currencyUK.string(11.12))
        assertEquals("£12,000,123.00", currencyUK.string(12_000_123.0))
    }

    @Test
    fun fiat_alwaysTwoPlaces() {
        assertEquals("$0.50", fiatUS.string(0.5))
        assertEquals("$0.00", fiatUS.string(0.0001234))
        assertEquals("$1,234.56", fiatUS.string(1234.56))
        assertEquals("$11.00", fiatUS.string(11.0))
    }

    @Test
    fun bigDecimalInput() {
        assertEquals("$1,234.56", currencyUS.string(BigDecimal("1234.56")))
        assertEquals("$0.0001234", currencyUS.string(BigDecimal("0.0001234")))
    }
}
