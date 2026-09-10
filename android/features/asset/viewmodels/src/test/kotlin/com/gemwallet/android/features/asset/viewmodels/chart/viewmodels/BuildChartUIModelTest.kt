package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.from
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemChart
import uniffi.gemstone.GemChartCurrent

class BuildChartUIModelTest {

    @Test
    fun `empty chart renders nothing`() {
        val model = ChartUIModel.from(chart = chart(emptyList()), period = ChartPeriod.Day, currency = Currency.USD)
        assertTrue(model.chartPoints.isEmpty())
        assertNull(model.currentPoint)
    }

    @Test
    fun `the current point core supplies is appended after history with its own change`() {
        val model = ChartUIModel.from(
            chart = chart(listOf(value(1_000L)), current = GemChartCurrent(date = 2_000L, value = 200.0, changePercentage = 4.2)),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertNotNull(model.currentPoint)
        assertEquals(2, model.chartPoints.size)
        assertEquals(200.0, model.chartPoints.last().price, 0.0)
        assertEquals(4.2, model.currentPoint!!.priceChangePercentage, 0.0001)

        val withoutCurrent = ChartUIModel.from(chart = chart(listOf(value(1_000L))), period = ChartPeriod.Day, currency = Currency.USD)
        assertNull(withoutCurrent.currentPoint)
        assertEquals(1, withoutCurrent.chartPoints.size)
    }

    @Test
    fun `history points change against the base core supplies`() {
        val model = ChartUIModel.from(
            chart = chart(listOf(value(1L, 0.0), value(2L, 100.0), value(3L, 150.0)), baseValue = 100.0),
            period = ChartPeriod.Week,
            currency = Currency.USD,
        )
        assertEquals(listOf(-100.0, 0.0, 50.0), model.chartPoints.map { it.priceChangePercentage })
    }

    @Test
    fun `render points match chart points count`() {
        val model = ChartUIModel.from(
            chart = chart(listOf(1.38, 1.37, 1.39, 1.38).mapIndexed { index, price -> value(index.toLong(), price) }),
            period = ChartPeriod.Hour,
            currency = Currency.USD,
        )
        assertEquals(4, model.renderPoints.size)
        assertEquals(0f, model.renderPoints.first().x)
        assertEquals(3f, model.renderPoints.last().x)
    }

    @Test
    fun `min and max labels resolved correctly`() {
        val model = ChartUIModel.from(
            chart = chart(listOf(1.38, 1.35, 1.42, 1.39).mapIndexed { index, price -> value(index.toLong(), price) }),
            period = ChartPeriod.Hour,
            currency = Currency.USD,
        )
        assertEquals("$1.35", model.minLabel)
        assertEquals("$1.42", model.maxLabel)
    }

    private fun value(date: Long, price: Double = 100.0) = ChartDateValue(date = date, value = price)

    private fun chart(values: List<ChartDateValue>, baseValue: Double = values.firstOrNull()?.value ?: 0.0, current: GemChartCurrent? = null) =
        GemChart(values = values.map { it.toGem() }, baseValue = baseValue, current = current)
}
