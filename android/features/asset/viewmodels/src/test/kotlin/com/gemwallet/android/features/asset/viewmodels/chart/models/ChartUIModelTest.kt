package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.testkit.mockFormattedNumber
import com.gemwallet.android.ui.components.chart.ChartBoundLabel
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.GemChartBounds
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartValueType
import uniffi.gemstone.GemChartViewport

class ChartUIModelTest {

    private val values = (0..4).map { ChartDateValue(date = it * 1_000L, value = 10.0 + it) }
    private val low = mockFormattedNumber(11.0)
    private val high = mockFormattedNumber(14.0)

    private val model = ChartUIModel(
        chart = GemChartData(valueType = GemChartValueType.PRICE, base = 10.0, showsSecondaryValue = false, currency = Currency.USD.toGem(), values = values, header = null),
        viewport = GemChartViewport(
            start = 1_000L,
            end = 4_000L,
            values = values.drop(1),
            renderValues = values,
            bounds = GemChartBounds(lowerIndex = 0u, upperIndex = 3u, yMin = 10.0, yMax = 15.0, low = low, high = high),
        ),
    )

    @Test
    fun `points sit at their time in the zoom window`() {
        assertEquals(listOf(-1f / 3, 0f, 1f / 3, 2f / 3, 1f), model.points.map { it.x })
        assertEquals(listOf(10f, 11f, 12f, 13f, 14f), model.points.map { it.y })
    }

    @Test
    fun `scrubbing starts at the first point inside the window`() {
        assertEquals(1, model.selectableFrom)
        assertNull("the point the line enters from is never selected", model.header(0))
    }

    @Test
    fun `the low and high labels sit at their points in the window`() {
        assertEquals(ChartBoundLabel(x = 0f, text = low.text()), model.minLabel)
        assertEquals(ChartBoundLabel(x = 1f, text = high.text()), model.maxLabel)
    }
}
