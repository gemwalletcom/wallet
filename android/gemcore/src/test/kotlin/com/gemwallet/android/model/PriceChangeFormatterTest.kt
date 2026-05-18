package com.gemwallet.android.model

import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.Locale

class PriceChangeFormatterTest {

    private val fiatUS = PriceChangeFormatter(
        CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD, locale = Locale.US)
    )
    private val adaptiveUS = PriceChangeFormatter(
        CurrencyFormatter(currency = Currency.USD, locale = Locale.US)
    )

    @Test
    fun string() {
        assertEquals("+$1.23", fiatUS.string(1.23))
        assertEquals("+$1,234.50", fiatUS.string(1234.5))
        assertEquals("-$1.23", fiatUS.string(-1.23))
        assertEquals("-$1,234.50", fiatUS.string(-1234.5))
        assertEquals("+$0.00", fiatUS.string(0.0))
        assertEquals("+$0.7006", adaptiveUS.string(0.7006))
        assertEquals("-$0.7006", adaptiveUS.string(-0.7006))
        assertEquals("+$140.57", adaptiveUS.string(140.5699884368446))
    }
}
