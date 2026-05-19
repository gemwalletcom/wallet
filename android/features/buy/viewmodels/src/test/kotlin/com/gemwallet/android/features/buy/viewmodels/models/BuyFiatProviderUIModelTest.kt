package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockFiatQuote
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatQuoteType
import org.junit.Assert.assertEquals
import org.junit.Test

class BuyFiatProviderUIModelTest {

    private val testAsset = mockAsset()
    private val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

    @Test
    fun `buy fiatFormatted is asset price times crypto amount`() {
        val quote = mockFiatQuote(fiatAmount = 50.0, cryptoAmount = 0.000488)

        val model = quote.toProviderUIModel(testAsset, Currency.USD, assetPrice = 100000.0)

        assertEquals(formatter.string(48.8), model.fiatFormatted)
    }

    @Test
    fun `buy fiatFormatted falls back to raw fiat amount when asset price is missing`() {
        val quote = mockFiatQuote(fiatAmount = 50.0, cryptoAmount = 0.000488)

        assertEquals(formatter.string(50.0), quote.toProviderUIModel(testAsset, Currency.USD, null).fiatFormatted)
        assertEquals(formatter.string(50.0), quote.toProviderUIModel(testAsset, Currency.USD, 0.0).fiatFormatted)
    }

    @Test
    fun `sell fiatFormatted ignores asset price and uses raw fiat amount`() {
        val quote = mockFiatQuote(type = FiatQuoteType.Sell, fiatAmount = 48.0, cryptoAmount = 0.001)

        val model = quote.toProviderUIModel(testAsset, Currency.USD, assetPrice = 100000.0)

        assertEquals(formatter.string(48.0), model.fiatFormatted)
    }
}
