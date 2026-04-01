package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockFiatProvider
import com.gemwallet.android.testkit.mockFiatQuote
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class BuyFiatProviderUIModelTest {

    private val testAsset = mockAsset()
    private val testProvider = mockFiatProvider()

    @Test
    fun `toProviderUIModel maps quote fields correctly`() {
        val model = mockFiatQuote().toProviderUIModel(testAsset, Currency.USD)

        assertEquals(testProvider, model.provider)
        assertEquals(testAsset, model.asset)
        assertEquals(0.17, model.cryptoAmount, 0.001)
    }

    @Test
    fun `cryptoFormatted starts with approximately symbol`() {
        val model = mockFiatQuote().toProviderUIModel(testAsset, Currency.USD)
        assertTrue(model.cryptoFormatted.startsWith("≈"))
    }

    @Test
    fun `cryptoText contains asset symbol`() {
        val model = mockFiatQuote().toProviderUIModel(testAsset, Currency.USD)
        assertTrue(model.cryptoText.contains(testAsset.symbol))
    }

    @Test
    fun `fiatFormatted is populated`() {
        val model = mockFiatQuote().toProviderUIModel(testAsset, Currency.USD)
        assertTrue(model.fiatFormatted.isNotEmpty())
    }

    @Test
    fun `rate text contains asset symbol and approximately`() {
        val model = mockFiatQuote().toProviderUIModel(testAsset, Currency.USD)
        assertTrue(model.rate.contains(testAsset.symbol))
        assertTrue(model.rate.contains("≈"))
    }
}
