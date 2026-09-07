package com.gemwallet.android.model

import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Currency
import junit.framework.TestCase.assertEquals
import org.junit.Test
import java.math.BigDecimal
import java.math.BigInteger
import java.util.Locale

class TestFormatter {
    @Test
    fun testCompactFormat_Italy() {
        val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.EUR, locale = Locale.ITALY)
        assertEquals("5\u00A0Mln\u00A0€", formatter.string(5_000_000.0))
        assertEquals("7,89\u00A0Mld\u00A0€", formatter.string(7_890_000_000.0))
        assertEquals("1,2\u00A0Bln\u00A0€", formatter.string(1_200_000_000_000.0))
    }

    @Test
    fun testCompactFormat_Usd() {
        val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.USD, locale = Locale.US)
        assertEquals("\$5M", formatter.string(5_000_000.0))
        assertEquals("\$7.89B", formatter.string(7_890_000_000.0))
        assertEquals("\$1.2T", formatter.string(1_200_000_000_000.0))
        assertEquals("\$19.87M", formatter.string(1.9876725E7))
    }

    @Test
    fun testCompactBalance_Usd() {
        val formatter = ValueFormatter(style = ValueFormatter.Style.Short, locale = Locale.US)
        assertEquals("123.45K USDC", formatter.string(BigInteger.valueOf(123_456_789_100L), decimals = 6, currency = "USDC"))
        assertEquals("1.5M USDC", formatter.string(BigInteger.valueOf(1_500_000_000_000L), decimals = 6, currency = "USDC"))
    }

    @Test
    fun testCompactKeepsDigitsWithoutRounding() {
        val formatter = ValueFormatter(style = ValueFormatter.Style.Short, locale = Locale.US)
        assertEquals("267.12K BTC", formatter.string(BigDecimal("267123.456"), currency = "BTC"))
        assertEquals("20.07M BTC", formatter.string(BigDecimal("20070000"), currency = "BTC"))
        assertEquals("19.87M BTC", formatter.string(BigDecimal("19876725"), currency = "BTC"))

        val currencyFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.USD, locale = Locale.US)
        assertEquals("\$267.12K", currencyFormatter.string(267_123.0))
    }
}
