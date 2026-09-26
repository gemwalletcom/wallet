package com.gemwallet.android.features.fiat_connect.viewmodels.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockGemFiatQuoteRow
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatProviderName
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.formattedAmount
import uniffi.gemstone.formattedCurrency

class FiatQuoteUIModelTest {

    @Test
    fun `the row's fiat amount is the one shown`() {
        val model = mockGemFiatQuoteRow(
            quoteId = "quote-1",
            provider = FiatProviderName.Mercuryo.toGem(),
            providerName = "Mercuryo",
            cryptoAmount = formattedAmount(0.17, "BTC", GemValueStyle.AUTO),
            fiatAmount = formattedCurrency(48.8, "USD", GemCurrencyStyle.FIAT),
        ).toQuoteUIModel()

        assertEquals(formattedCurrency(48.8, Currency.USD.string, GemCurrencyStyle.FIAT).text(), model.fiatFormatted)
    }

    @Test
    fun `cryptoText keeps significant digits for amounts sharing four decimals`() {
        val first = mockGemFiatQuoteRow(
            quoteId = "quote-1",
            provider = FiatProviderName.Mercuryo.toGem(),
            providerName = "Mercuryo",
            cryptoAmount = formattedAmount(0.00077, "BTC", GemValueStyle.AUTO),
            fiatAmount = formattedCurrency(100.0, "USD", GemCurrencyStyle.FIAT),
        ).toQuoteUIModel()
        val second = mockGemFiatQuoteRow(
            quoteId = "quote-1",
            provider = FiatProviderName.Mercuryo.toGem(),
            providerName = "Mercuryo",
            cryptoAmount = formattedAmount(0.0007578, "BTC", GemValueStyle.AUTO),
            fiatAmount = formattedCurrency(100.0, "USD", GemCurrencyStyle.FIAT),
        ).toQuoteUIModel()

        assertNotEquals(first.cryptoText, second.cryptoText)
    }
}
