package com.gemwallet.android.features.confirm.models

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemAutocloseSummary

class PerpetualModifyAutocloseFactoryTest {

    @Test
    fun formatsPricesAndDashesClearedOrders() {
        val prices = PerpetualModifyAutocloseFactory.element(GemAutocloseSummary(65000.0, 55000.0, false, false))
        assertEquals("$65,000.00", prices.takeProfitText)
        assertEquals("$55,000.00", prices.stopLossText)

        val cleared = PerpetualModifyAutocloseFactory.element(GemAutocloseSummary(null, null, true, true))
        assertEquals("-", cleared.takeProfitText)
        assertEquals("-", cleared.stopLossText)

        val onlyTakeProfit = PerpetualModifyAutocloseFactory.element(GemAutocloseSummary(70000.0, null, false, false))
        assertEquals("$70,000.00", onlyTakeProfit.takeProfitText)
        assertNull(onlyTakeProfit.stopLossText)
    }
}
