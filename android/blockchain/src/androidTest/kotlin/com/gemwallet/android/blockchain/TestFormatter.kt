package com.gemwallet.android.blockchain

import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.wallet.core.primitives.Currency
import junit.framework.TestCase.assertEquals
import org.junit.Test
import java.math.BigInteger
import java.util.Locale

class TestFormatter {
    @Test
    fun testCompactFormat_Italy() {
        val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.EUR, locale = Locale.ITALY)
        assertEquals("5 Mio €", formatter.string(5_000_000.0))
        assertEquals("7,9 Mrd €", formatter.string(7_890_000_000.0))
        assertEquals("1,2 Bln €", formatter.string(1_200_000_000_000.0))
    }

    @Test
    fun testCompactFormat_Usd() {
        val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Abbreviated, currency = Currency.USD, locale = Locale.US)
        assertEquals("\$5M", formatter.string(5_000_000.0))
        assertEquals("\$7.9B", formatter.string(7_890_000_000.0))
        assertEquals("\$1.2T", formatter.string(1_200_000_000_000.0))
        assertEquals("\$20M", formatter.string(1.9876725E7))
    }

    @Test
    fun testCompactBalance_Usd() {
        val formatter = ValueFormatter(style = ValueFormatter.Style.Short, locale = Locale.US)
        assertEquals("120K USDC", formatter.string(BigInteger.valueOf(123_456_789_100L), decimals = 6, currency = "USDC"))
        assertEquals("1.5M USDC", formatter.string(BigInteger.valueOf(1_500_000_000_000L), decimals = 6, currency = "USDC"))
    }
}
