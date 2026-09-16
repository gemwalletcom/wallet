package com.gemwallet.android.features.confirm.models

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemAutocloseSummary
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.formattedCurrency

class PerpetualModifyAutocloseFactoryTest {

    private fun usd(value: Double) = formattedCurrency(value, Currency.USD.string, GemCurrencyStyle.CURRENCY)

    @Test
    fun formatsPricesAndDashesClearedOrders() {
        val prices = PerpetualModifyAutocloseFactory.element(GemAutocloseSummary(usd(65000.0), usd(55000.0), false, false))
        assertEquals("$65,000.00", prices.takeProfitText)
        assertEquals("$55,000.00", prices.stopLossText)

        val cleared = PerpetualModifyAutocloseFactory.element(GemAutocloseSummary(null, null, true, true))
        assertEquals("-", cleared.takeProfitText)
        assertEquals("-", cleared.stopLossText)

        val onlyTakeProfit = PerpetualModifyAutocloseFactory.element(GemAutocloseSummary(usd(70000.0), null, false, false))
        assertEquals("$70,000.00", onlyTakeProfit.takeProfitText)
        assertNull(onlyTakeProfit.stopLossText)
    }
}
