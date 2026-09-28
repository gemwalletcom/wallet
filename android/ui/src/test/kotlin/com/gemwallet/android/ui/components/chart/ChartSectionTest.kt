package com.gemwallet.android.ui.components.chart

import com.gemwallet.android.testkit.mockGemChartBounds
import com.gemwallet.android.testkit.mockGemChartData
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.ChartDateValue

class ChartSectionTest {

    private val chart = mockGemChartData(
        values = (0..4).map { ChartDateValue(date = it * 1_000L, value = 10.0 + it) },
        bounds = mockGemChartBounds(lowerIndex = 1u, upperIndex = 4u),
        start = 1_000L,
        end = 4_000L,
    )

    @Test
    fun `points sit at their time in the zoom window`() {
        assertEquals(listOf(-1f / 3, 0f, 1f / 3, 2f / 3, 1f), chart.linePoints().map { it.x })
        assertEquals(listOf(10f, 11f, 12f, 13f, 14f), chart.linePoints().map { it.y })
    }

    @Test
    fun `scrubbing starts at the first point inside the window`() {
        assertEquals(1, chart.selectableFrom())
    }

    @Test
    fun `the low and high labels sit at their points in the window`() {
        assertEquals(ChartBoundLabel(x = 0f, text = "low"), chart.boundLabel(chart.bounds.lowerIndex, "low"))
        assertEquals(ChartBoundLabel(x = 1f, text = "high"), chart.boundLabel(chart.bounds.upperIndex, "high"))
        assertNull(chart.boundLabel(9u, "missing"))
    }
}
