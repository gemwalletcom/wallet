package com.gemwallet.android.domains.pricealerts

import com.wallet.core.primitives.PriceAlertDirection
import com.wallet.core.primitives.PriceAlertNotificationType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PriceAlertNotificationTypeExtTest {

    @Test
    fun `price direction is null when target equals current price`() {
        assertNull(PriceAlertNotificationType.Price.direction(currentPrice = 100.0, inputValue = 100.0, selectedDirection = PriceAlertDirection.Up))
    }

    @Test
    fun `price direction is null when there is no current price yet`() {
        assertNull(PriceAlertNotificationType.Price.direction(currentPrice = 0.0, inputValue = 100.0, selectedDirection = PriceAlertDirection.Up))
    }

    @Test
    fun `price direction is up when target is above current price`() {
        assertEquals(
            PriceAlertDirection.Up,
            PriceAlertNotificationType.Price.direction(currentPrice = 100.0, inputValue = 150.0, selectedDirection = PriceAlertDirection.Down),
        )
    }

    @Test
    fun `price direction is down when target is below current price`() {
        assertEquals(
            PriceAlertDirection.Down,
            PriceAlertNotificationType.Price.direction(currentPrice = 100.0, inputValue = 50.0, selectedDirection = PriceAlertDirection.Up),
        )
    }

    @Test
    fun `percent change direction always follows the selected direction`() {
        assertEquals(
            PriceAlertDirection.Down,
            PriceAlertNotificationType.PricePercentChange.direction(currentPrice = 0.0, inputValue = 5.0, selectedDirection = PriceAlertDirection.Down),
        )
    }
}
