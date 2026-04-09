package com.gemwallet.android.features.settings.price_alerts.presents

import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertType
import com.gemwallet.android.ui.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PriceAlertSupportContentTest {

    @Test
    fun autoAlert_usesFormattedPriceAndPercentage() {
        val content = priceAlertSupportContent(
            type = PriceAlertType.Auto,
            price = "$0.95",
            percentage = "+10.04%",
        )

        assertNull(content.labelRes)
        assertEquals("$0.95", content.primaryText)
        assertEquals("+10.04%", content.secondaryText)
        assertTrue(content.hasContent)
    }

    @Test
    fun overAlert_usesOverLabelAndTargetPrice() {
        val content = priceAlertSupportContent(
            type = PriceAlertType.Over,
            price = "$1.25",
            percentage = "+3.50%",
        )

        assertEquals(R.string.price_alerts_direction_over, content.labelRes)
        assertEquals("", content.primaryText)
        assertEquals("$1.25", content.secondaryText)
    }

    @Test
    fun underAlert_usesUnderLabelAndTargetPrice() {
        val content = priceAlertSupportContent(
            type = PriceAlertType.Under,
            price = "$0.40",
            percentage = "-1.50%",
        )

        assertEquals(R.string.price_alerts_direction_under, content.labelRes)
        assertEquals("", content.primaryText)
        assertEquals("$0.40", content.secondaryText)
    }

    @Test
    fun increaseAlert_usesIncreaseLabelAndTargetPercentage() {
        val content = priceAlertSupportContent(
            type = PriceAlertType.Increase,
            price = "$2.10",
            percentage = "5.00%",
        )

        assertEquals(R.string.price_alerts_direction_increases_by, content.labelRes)
        assertEquals("", content.primaryText)
        assertEquals("5.00%", content.secondaryText)
    }

    @Test
    fun decreaseAlert_usesDecreaseLabelAndTargetPercentage() {
        val content = priceAlertSupportContent(
            type = PriceAlertType.Decrease,
            price = "$2.10",
            percentage = "5.00%",
        )

        assertEquals(R.string.price_alerts_direction_decreases_by, content.labelRes)
        assertEquals("", content.primaryText)
        assertEquals("5.00%", content.secondaryText)
    }
}
