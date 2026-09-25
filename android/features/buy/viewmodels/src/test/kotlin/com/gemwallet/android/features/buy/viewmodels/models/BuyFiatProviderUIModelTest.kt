package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockFiatQuoteRow
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.formattedCurrency

class BuyFiatProviderUIModelTest {

    @Test
    fun `the row's fiat amount is the one shown`() {
        val model = mockFiatQuoteRow(fiatAmount = 48.8).toProviderUIModel()

        assertEquals(formattedCurrency(48.8, Currency.USD.string, GemCurrencyStyle.FIAT).text(), model.fiatFormatted)
    }

    @Test
    fun `cryptoText keeps significant digits for amounts sharing four decimals`() {
        val first = mockFiatQuoteRow(cryptoAmount = 0.00077).toProviderUIModel()
        val second = mockFiatQuoteRow(cryptoAmount = 0.0007578).toProviderUIModel()

        assertNotEquals(first.cryptoText, second.cryptoText)
    }
}
