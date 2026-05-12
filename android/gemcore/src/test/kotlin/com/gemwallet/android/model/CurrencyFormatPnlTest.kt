package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.Locale

class CurrencyFormatPnlTest {

    @Test
    fun formatPnl_positive_prependsPlusSign() {
        Locale.setDefault(Locale.US)
        assertEquals("+\$1.23", Currency.USD.formatPnl(1.23))
        assertEquals("+\$1,234.50", Currency.USD.formatPnl(1234.5))
    }

    @Test
    fun formatPnl_negative_prependsMinusOnAbsoluteValue() {
        Locale.setDefault(Locale.US)
        assertEquals("-\$1.23", Currency.USD.formatPnl(-1.23))
        assertEquals("-\$1,234.50", Currency.USD.formatPnl(-1234.5))
    }

    @Test
    fun formatPnl_zero_prependsPlusSign() {
        Locale.setDefault(Locale.US)
        assertEquals("+\$0.00", Currency.USD.formatPnl(0.0))
    }

    @Test
    fun formatPnl_dynamicPlace_usesSmallValuePrecision() {
        Locale.setDefault(Locale.US)
        assertEquals("-\$0.7006", Currency.USD.formatPnl(-0.7006, dynamicPlace = true))
        assertEquals("+\$0.7006", Currency.USD.formatPnl(0.7006, dynamicPlace = true))
    }
}
