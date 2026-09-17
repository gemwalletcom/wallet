package com.gemwallet.android.domains.price.values

import uniffi.gemstone.GemValueTone
import com.wallet.core.primitives.Currency
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertSame
import org.junit.Before
import org.junit.Test
import java.util.Locale

class RowFormattersTest {
    private val defaultLocale = Locale.getDefault()

    @Before
    fun setup() {
        Locale.setDefault(Locale.US)
    }

    @After
    fun tearDown() {
        Locale.setDefault(defaultLocale)
    }

    @Test
    fun price_formatsValueChangeAndDirectionOnce() {
        val price = RowFormatters().price(Currency.USD, 50000.0, 2.5)

        assertEquals("$50,000.00", price.valueFormatted)
        assertEquals("+2.50%", price.changePercentageFormatted)
        assertEquals(GemValueTone.POSITIVE, price.state)
        assertEquals(50000.0, price.value)
    }

    @Test
    fun price_dropsNonFiniteNumbers() {
        val price = RowFormatters().price(Currency.EUR, Double.NaN, Double.POSITIVE_INFINITY)

        assertEquals(null, price.value)
        assertEquals(null, price.changePercentage)
        assertEquals("", price.valueFormatted)
        assertEquals("", price.changePercentageFormatted)
        assertEquals(GemValueTone.NEUTRAL, price.state)
    }

    @Test
    fun formattersAreReusedPerCurrency() {
        val formatters = RowFormatters()

        assertSame(formatters.currency(Currency.USD), formatters.currency(Currency.USD))
        assertEquals("$1,234.50", formatters.currency(Currency.USD).string(1234.5))
    }
}
