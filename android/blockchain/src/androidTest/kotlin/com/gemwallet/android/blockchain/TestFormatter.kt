package com.gemwallet.android.blockchain

import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Currency
import junit.framework.TestCase.assertEquals
import org.junit.Test
import java.util.Locale

class TestFormatter {
    @Test
    fun testCompactFormat_Italy() {
        val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.EUR, locale = Locale.ITALY)
        assertEquals("5,00 Mio €", formatter.string(5_000_000.0))
        assertEquals("7,89 Mrd €", formatter.string(7_890_000_000.0))
        assertEquals("1,20 Bln €", formatter.string(1_200_000_000_000.0))
    }

    @Test
    fun testCompactFormat_Usd() {
        val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.USD, locale = Locale.US)
        assertEquals("\$5.00M", formatter.string(5_000_000.0))
        assertEquals("\$7.89B", formatter.string(7_890_000_000.0))
        assertEquals("\$1.20T", formatter.string(1_200_000_000_000.0))
        assertEquals("\$19.88M", formatter.string(1.9876725E7))
    }
}
