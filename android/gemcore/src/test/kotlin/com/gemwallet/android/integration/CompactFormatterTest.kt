package com.gemwallet.android.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Currency
import junit.framework.TestCase.assertEquals
import junit.framework.TestCase.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemValueStyle
import java.math.BigDecimal
import java.math.BigInteger
import java.util.Locale

@RunWith(AndroidJUnit4::class)
class CompactFormatterTest {

    private fun assertCompact(value: String, formatted: String) {
        assertTrue("$formatted starts with $value", formatted.startsWith("$value\u00A0"))
        assertTrue("$formatted ends with the currency", formatted.endsWith("\u00A0€"))
    }

    @Test
    fun testCompactFormat_Italy() {
        val formatter = CurrencyFormatter(style = GemCurrencyStyle.ABBREVIATED, currency = Currency.EUR, locale = Locale.ITALY)
        assertCompact("5", formatter.string(5_000_000.0))
        assertCompact("7,89", formatter.string(7_890_000_000.0))
        assertCompact("1,2", formatter.string(1_200_000_000_000.0))
    }

    @Test
    fun testCompactFormat_Usd() {
        val formatter = CurrencyFormatter(style = GemCurrencyStyle.ABBREVIATED, currency = Currency.USD, locale = Locale.US)
        assertEquals("\$5M", formatter.string(5_000_000.0))
        assertEquals("\$7.89B", formatter.string(7_890_000_000.0))
        assertEquals("\$1.2T", formatter.string(1_200_000_000_000.0))
        assertEquals("\$19.88M", formatter.string(1.9876725E7))
    }

    @Test
    fun testCompactBalance_Usd() {
        val formatter = ValueFormatter(style = GemValueStyle.SHORT, locale = Locale.US)
        assertEquals("123.45K USDC", formatter.string(BigInteger.valueOf(123_456_789_100L), decimals = 6, currency = "USDC"))
        assertEquals("1.5M USDC", formatter.string(BigInteger.valueOf(1_500_000_000_000L), decimals = 6, currency = "USDC"))
    }

    @Test
    fun testCompactKeepsDigitsWithoutRounding() {
        val formatter = ValueFormatter(style = GemValueStyle.SHORT, locale = Locale.US)
        assertEquals("267.12K BTC", formatter.string(BigDecimal("267123.456"), currency = "BTC"))
        assertEquals("20.07M BTC", formatter.string(BigDecimal("20070000"), currency = "BTC"))
        assertEquals("19.87M BTC", formatter.string(BigDecimal("19876725"), currency = "BTC"))

        val currencyFormatter = CurrencyFormatter(style = GemCurrencyStyle.ABBREVIATED, currency = Currency.USD, locale = Locale.US)
        assertEquals("\$267.12K", currencyFormatter.string(267_123.0))
    }
}
