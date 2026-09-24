package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.testkit.mockFiatQuoteRow
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import uniffi.gemstone.GemCurrencyStyle

class BuyFiatProviderUIModelTest {

    private val formatter = CurrencyFormatter(style = GemCurrencyStyle.FIAT, currency = Currency.USD)

    @Test
    fun `the row's fiat amount is the one shown`() {
        val model = mockFiatQuoteRow(fiatAmount = 48.8).toProviderUIModel()

        assertEquals(formatter.string(48.8), model.fiatFormatted)
    }

    @Test
    fun `cryptoText keeps significant digits for amounts sharing four decimals`() {
        val first = mockFiatQuoteRow(cryptoAmount = 0.00077).toProviderUIModel()
        val second = mockFiatQuoteRow(cryptoAmount = 0.0007578).toProviderUIModel()

        assertNotEquals(first.cryptoText, second.cryptoText)
    }
}
