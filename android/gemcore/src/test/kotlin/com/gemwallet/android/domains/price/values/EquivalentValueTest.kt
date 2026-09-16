package com.gemwallet.android.domains.price.values

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.testkit.mockEquivalentValue
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class EquivalentValueTest {

    @Test
    fun testPriceableValue_stateUp() {
        val price = mockEquivalentValue(changePercentage = 2.5)

        assertEquals(ValueDirection.Up, price.state)
    }

    @Test
    fun testPriceableValue_stateDown() {
        val price = mockEquivalentValue(changePercentage = -2.5)

        assertEquals(ValueDirection.Down, price.state)
    }

    @Test
    fun testPriceableValue_stateNone() {
        val price = mockEquivalentValue(changePercentage = 0.0)

        assertEquals(ValueDirection.None, price.state)
    }

    @Test
    fun testPriceableValue_stateNullPercentage() {
        val price = mockEquivalentValue(changePercentage = null)

        assertEquals(ValueDirection.None, price.state)
    }

    @Test
    fun testPriceableValue_dayChangePercentageFormattedPositive() {
        val price = mockEquivalentValue(changePercentage = 2.5)
        val formatted = price.changePercentageFormatted

        assertEquals("+2.50%", formatted)
    }

    @Test
    fun testPriceableValue_dayChangePercentageFormattedNegative() {
        val price = mockEquivalentValue(changePercentage = -5.2)
        val formatted = price.changePercentageFormatted

        assertEquals("-5.20%", formatted)
    }

    @Test
    fun testPriceableValue_dayChangePercentageFormattedZero() {
        val price = mockEquivalentValue(changePercentage = 0.0)
        val formatted = price.changePercentageFormatted

        assertEquals("+0.00%", formatted)
    }

    @Test
    fun testPriceableValue_dayChangePercentageFormattedNull() {
        val price = mockEquivalentValue(changePercentage = null)
        val formatted = price.changePercentageFormatted

        assertEquals("", formatted)
    }

    @Test
    fun testPriceableValue_priceValueFormattedWithNull() {
        val price = mockEquivalentValue(value = null)
        val formatted = price.valueFormatted

        assertEquals("", formatted)
    }

    @Test
    fun testPriceableValue_priceValueFormattedWithNaN() {
        val price = mockEquivalentValue(value = Double.NaN)
        val formatted = price.valueFormatted

        assertEquals("", formatted)
    }

    @Test
    fun testPriceableValue_priceValueFormattedWithInfinity() {
        val price = mockEquivalentValue(value = Double.POSITIVE_INFINITY)
        val formatted = price.valueFormatted

        assertEquals("", formatted)
    }

    @Test
    fun testPriceableValue_priceValueFormattedWithZero() {
        val price = mockEquivalentValue(value = 0.0)
        val formatted = price.valueFormatted

        assertTrue("Formatted zero price should not be empty", formatted.isNotEmpty())
    }

    @Test
    fun testPriceableValue_priceValueFormattedWithLowValue() {
        val price = mockEquivalentValue(value = 0.0025)
        val formatted = price.valueFormatted

        assertTrue("Formatted low price should not be empty", formatted.isNotEmpty())
    }

    @Test
    fun testPriceableValue_priceValueFormattedWithNegative() {
        val price = mockEquivalentValue(value = -100.0)
        val formatted = price.valueFormatted

        assertTrue("Formatted negative price should not be empty", formatted.isNotEmpty())
    }

    @Test
    fun testPriceableValue_multiplePropertiesConsistency() {
        val price = mockEquivalentValue(
            value = 50000.0,
            changePercentage = 3.5,
            currency = Currency.USD
        )

        assertEquals(50000.0, price.value!!, 0.01)
        assertEquals(3.5, price.changePercentage!!, 0.01)
        assertEquals(Currency.USD, price.currency)
        assertEquals(ValueDirection.Up, price.state)
        assertEquals("+3.50%", price.changePercentageFormatted)
    }

    @Test
    fun testPriceableValue_stateTransitionFromPositiveToNegative() {
        val priceUp = mockEquivalentValue(changePercentage = 1.5)
        assertEquals(ValueDirection.Up, priceUp.state)

        val priceDown = mockEquivalentValue(changePercentage = -1.5)
        assertEquals(ValueDirection.Down, priceDown.state)
    }

    @Test
    fun testPriceableValue_stateWithVerySmallChange() {
        val priceSmallUp = mockEquivalentValue(changePercentage = 0.001)
        assertEquals(ValueDirection.Up, priceSmallUp.state)
        assertEquals("+0.00%", priceSmallUp.changePercentageFormatted)

        val priceSmallDown = mockEquivalentValue(changePercentage = -0.001)
        assertEquals(ValueDirection.Down, priceSmallDown.state)
        assertEquals("-0.00%", priceSmallDown.changePercentageFormatted)
    }

    @Test
    fun testPriceableValue_stateWithMinimumDetectableChange() {
        val priceMinUp = mockEquivalentValue(changePercentage = 0.01)
        assertEquals(ValueDirection.Up, priceMinUp.state)

        val priceMinDown = mockEquivalentValue(changePercentage = -0.01)
        assertEquals(ValueDirection.Down, priceMinDown.state)
    }
}
