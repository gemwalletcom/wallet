package com.gemwallet.android.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.model.text
import junit.framework.TestCase.assertEquals
import junit.framework.TestCase.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.formattedCurrency
import java.util.Locale

@RunWith(AndroidJUnit4::class)
class CompactFormatterTest {

    private fun currency(style: GemCurrencyStyle, code: String, locale: Locale): (Double) -> String = { formattedCurrency(it, code, style).text(locale) }

    private fun assertCompact(value: String, formatted: String) {
        assertTrue("$formatted starts with $value", formatted.startsWith("$value\u00A0"))
        assertTrue("$formatted ends with the currency", formatted.endsWith("\u00A0€"))
    }

    @Test
    fun testCompactFormat_Italy() {
        val formatter = currency(GemCurrencyStyle.ABBREVIATED, "EUR", Locale.ITALY)
        assertCompact("5", formatter(5_000_000.0))
        assertCompact("7,89", formatter(7_890_000_000.0))
        assertCompact("1,2", formatter(1_200_000_000_000.0))
    }

    @Test
    fun testCompactFormat_Usd() {
        val formatter = currency(GemCurrencyStyle.ABBREVIATED, "USD", Locale.US)
        assertEquals("\$5M", formatter(5_000_000.0))
        assertEquals("\$7.89B", formatter(7_890_000_000.0))
        assertEquals("\$1.2T", formatter(1_200_000_000_000.0))
        assertEquals("\$19.88M", formatter(1.9876725E7))
    }

    @Test
    fun testCompactKeepsDigitsWithoutRounding() {
        val formatter = currency(GemCurrencyStyle.ABBREVIATED, "USD", Locale.US)
        assertEquals("\$267.12K", formatter(267_123.0))
    }
}
