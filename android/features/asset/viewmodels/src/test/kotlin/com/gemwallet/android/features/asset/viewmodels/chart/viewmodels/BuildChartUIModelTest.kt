package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.features.asset.viewmodels.chart.models.Chart
import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.from
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class BuildChartUIModelTest {

    @Test
    fun `empty chart renders nothing`() {
        val model = ChartUIModel.from(
            chart = Chart(values = emptyList(), current = null),
            priceInfo = mockAssetPriceInfo(),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertTrue(model.chartPoints.isEmpty())
        assertNull(model.currentPoint)
    }

    @Test
    fun `the current point core supplies is appended after history`() {
        val model = ChartUIModel.from(
            chart = Chart(values = listOf(value(1_000L)), current = value(2_000L, 200.0)),
            priceInfo = mockAssetPriceInfo(updatedAt = 2_000L),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertNotNull(model.currentPoint)
        assertEquals(2, model.chartPoints.size)
        assertEquals(200.0, model.chartPoints.last().price, 0.0)

        val withoutCurrent = ChartUIModel.from(
            chart = Chart(values = listOf(value(1_000L)), current = null),
            priceInfo = mockAssetPriceInfo(updatedAt = 2_000L),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertNull(withoutCurrent.currentPoint)
        assertEquals(1, withoutCurrent.chartPoints.size)
    }

    @Test
    fun `day period uses 24h change for current point`() {
        val model = ChartUIModel.from(
            chart = Chart(values = listOf(value(1L)), current = value(2_000L, 200.0)),
            priceInfo = mockAssetPriceInfo(price = 200.0, priceChangePercentage24h = 4.2, updatedAt = 2_000L),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertEquals(4.2, model.currentPoint!!.priceChangePercentage, 0.0001)
    }

    @Test
    fun `non-day period calculates change from base price`() {
        val model = ChartUIModel.from(
            chart = Chart(values = listOf(value(1L, 100.0)), current = value(2_000L, 200.0)),
            priceInfo = mockAssetPriceInfo(price = 200.0, priceChangePercentage24h = 4.2, updatedAt = 2_000L),
            period = ChartPeriod.Week,
            currency = Currency.USD,
        )
        assertEquals(100.0, model.currentPoint!!.priceChangePercentage, 0.0001)
    }

    @Test
    fun `zero start price does not crash`() {
        val model = ChartUIModel.from(
            chart = Chart(values = listOf(value(1L, 0.0)), current = value(2_000L, 50.0)),
            priceInfo = mockAssetPriceInfo(price = 50.0, updatedAt = 2_000L),
            period = ChartPeriod.Week,
            currency = Currency.USD,
        )
        assertNotNull(model.currentPoint)
        assertEquals(0.0, model.currentPoint!!.priceChangePercentage, 0.0001)
    }

    @Test
    fun `render points match chart points count`() {
        val model = ChartUIModel.from(
            chart = Chart(values = listOf(1.38, 1.37, 1.39, 1.38).mapIndexed { index, price -> value(index.toLong(), price) }, current = null),
            priceInfo = null,
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
            chart = Chart(values = listOf(1.38, 1.35, 1.42, 1.39).mapIndexed { index, price -> value(index.toLong(), price) }, current = null),
            priceInfo = null,
            period = ChartPeriod.Hour,
            currency = Currency.USD,
        )
        assertEquals("$1.35", model.minLabel)
        assertEquals("$1.42", model.maxLabel)
    }

    private fun value(date: Long, price: Double = 100.0) = ChartDateValue(date = date, value = price)
}
