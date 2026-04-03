package com.gemwallet.android.ext

import com.gemwallet.android.testkit.mockPriceAlert
import com.wallet.core.primitives.PriceAlertDirection
import org.junit.Assert.assertEquals
import org.junit.Test

class PriceAlertExtTest {

    @Test
    fun `id matches rust generate_id`() {
        assertEquals("bitcoin", mockPriceAlert().id)
        assertEquals("bitcoin_USD_100_up", mockPriceAlert(price = 100.0, priceDirection = PriceAlertDirection.Up).id)
        assertEquals("bitcoin_USD_1.12344_down", mockPriceAlert(price = 1.12344, priceDirection = PriceAlertDirection.Down).id)
        assertEquals("bitcoin_USD_5_up", mockPriceAlert(pricePercentChange = 5.0, priceDirection = PriceAlertDirection.Up).id)
        assertEquals("bitcoin_USD_10000.1_down", mockPriceAlert(pricePercentChange = 10000.10, priceDirection = PriceAlertDirection.Down).id)
        assertEquals("bitcoin_USD_1_up", mockPriceAlert(price = 1.0, priceDirection = PriceAlertDirection.Up).id)
        assertEquals("bitcoin_USD_0.23", mockPriceAlert(pricePercentChange = 0.23).id)
        assertEquals("bitcoin_USD_50000.01", mockPriceAlert(price = 50000.01).id)
        assertEquals("bitcoin_USD_0.001234567", mockPriceAlert(price = 0.001234567).id)
    }
}
