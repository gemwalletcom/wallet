package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import java.util.Locale

class ChartHeaderUIModelTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    private val formatter: (Double) -> String = { "$%.2f".format(it) }

    @Test
    fun buildPopulatesFromRawValues() {
        val model = ChartHeaderUIModel.build(
            price = 110.0,
            priceChangePercentage = 10.0,
            timestamp = 5_000L,
            priceFormatter = formatter,
            dateFormatter = { "@$it" },
        )
        assertEquals("$110.00", model.priceText)
        assertEquals(ValueDirection.Up, model.direction)
        assertEquals("@5000", model.dateText)
        assertNull(model.headerValueText)
    }

    @Test
    fun buildOmitsDateWhenTimestampNull() {
        val model = ChartHeaderUIModel.build(
            price = 50.0,
            priceChangePercentage = -5.0,
            priceFormatter = formatter,
        )
        assertEquals("$50.00", model.priceText)
        assertEquals(ValueDirection.Down, model.direction)
        assertNull(model.dateText)
    }

    @Test
    fun buildFormatsHeaderValueWithPriceFormatter() {
        val model = ChartHeaderUIModel.build(
            price = 50.0,
            priceChangePercentage = 0.0,
            headerValue = 1500.0,
            priceFormatter = formatter,
        )
        assertEquals("$1500.00", model.headerValueText)
        assertEquals(ValueDirection.None, model.direction)
    }
}
