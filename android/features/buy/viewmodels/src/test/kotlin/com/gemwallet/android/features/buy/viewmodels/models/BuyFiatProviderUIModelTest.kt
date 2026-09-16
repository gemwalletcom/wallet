package com.gemwallet.android.features.buy.viewmodels.models

import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.formattedCurrency
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockFiatQuoteRow
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class BuyFiatProviderUIModelTest {

    private val testAsset = mockAsset()
    private val formatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

    @Test
    fun `the row's fiat amount is the one shown`() {
        val model = mockFiatQuoteRow(fiatAmount = 48.8).toProviderUIModel(testAsset)

        assertEquals(formatter.string(48.8), model.fiatFormatted)
    }

    @Test
    fun `a rate reads as one unit of the asset and is empty without one`() {
        val rate = GemAssetRate(baseSymbol = testAsset.symbol, quoteSymbol = Currency.USD.string, value = formattedCurrency(102500.0, Currency.USD.string, GemCurrencyStyle.CURRENCY))
        assertEquals("1 ${testAsset.symbol} ≈ ${formatter.string(102500.0)}", mockFiatQuoteRow(rate = rate).toProviderUIModel(testAsset).rate)
        assertEquals("", mockFiatQuoteRow(rate = null).toProviderUIModel(testAsset).rate)
    }

    @Test
    fun `cryptoText keeps significant digits for amounts sharing four decimals`() {
        val first = mockFiatQuoteRow(cryptoAmount = 0.00077).toProviderUIModel(testAsset)
        val second = mockFiatQuoteRow(cryptoAmount = 0.0007578).toProviderUIModel(testAsset)

        assertNotEquals(first.cryptoText, second.cryptoText)
    }
}
