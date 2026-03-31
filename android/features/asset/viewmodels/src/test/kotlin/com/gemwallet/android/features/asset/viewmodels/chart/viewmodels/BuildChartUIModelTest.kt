package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartUIModel
import com.gemwallet.android.features.asset.viewmodels.chart.models.from
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockChartPrices
import com.gemwallet.android.testkit.mockChartValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class BuildChartUIModelTest {

    @Test
    fun `empty prices returns empty chart`() {
        val model = ChartUIModel.from(
            prices = emptyList(),
            priceInfo = mockAssetPriceInfo(),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertTrue(model.chartPoints.isEmpty())
        assertNull(model.currentPoint)
    }

    @Test
    fun `current point appended when newer than last chart point`() {
        val lastTimestamp = 1000
        val model = ChartUIModel.from(
            prices = listOf(mockChartValue(timestamp = lastTimestamp)),
            priceInfo = mockAssetPriceInfo(updatedAt = lastTimestamp * 1000L + 1),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertNotNull(model.currentPoint)
        assertEquals(2, model.chartPoints.size)
    }

    @Test
    fun `current point not appended when older than last chart point`() {
        val lastTimestamp = 1000
        val model = ChartUIModel.from(
            prices = listOf(mockChartValue(timestamp = lastTimestamp)),
            priceInfo = mockAssetPriceInfo(updatedAt = lastTimestamp * 1000L - 1),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertNull(model.currentPoint)
        assertEquals(1, model.chartPoints.size)
    }

    @Test
    fun `day period uses 24h change for current point`() {
        val model = ChartUIModel.from(
            prices = listOf(mockChartValue(timestamp = 1)),
            priceInfo = mockAssetPriceInfo(price = 200.0, priceChangePercentage24h = 4.2, updatedAt = 2000L),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertEquals("+4.20%", model.currentPoint!!.percentage)
    }

    @Test
    fun `non-day period calculates change from base price`() {
        val model = ChartUIModel.from(
            prices = listOf(mockChartValue(timestamp = 1, value = 100.0f)),
            priceInfo = mockAssetPriceInfo(price = 200.0, priceChangePercentage24h = 4.2, updatedAt = 2000L),
            period = ChartPeriod.Week,
            currency = Currency.USD,
        )
        assertEquals("+100.00%", model.currentPoint!!.percentage)
    }

    @Test
    fun `zero start price does not crash`() {
        val model = ChartUIModel.from(
            prices = listOf(mockChartValue(timestamp = 1, value = 0.0f)),
            priceInfo = mockAssetPriceInfo(price = 50.0, updatedAt = 2000L),
            period = ChartPeriod.Week,
            currency = Currency.USD,
        )
        assertNotNull(model.currentPoint)
        assertEquals("0.00%", model.currentPoint!!.percentage)
    }

    @Test
    fun `render points match chart points count`() {
        val model = ChartUIModel.from(
            prices = mockChartPrices(values = listOf(1.38f, 1.37f, 1.39f, 1.38f)),
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
            prices = mockChartPrices(values = listOf(1.38f, 1.35f, 1.42f, 1.39f)),
            priceInfo = null,
            period = ChartPeriod.Hour,
            currency = Currency.USD,
        )
        assertEquals("$1.35", model.minLabel)
        assertEquals("$1.42", model.maxLabel)
    }
}
