package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.model.AssetPriceInfo
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class BuildChartUIModelTest {

    @Test
    fun `empty prices returns empty chart`() {
        val model = buildChartUIModel(
            prices = emptyList(),
            priceInfo = createPriceInfo(),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )

        assertTrue(model.chartPoints.isEmpty())
        assertNull(model.currentPoint)
    }

    @Test
    fun `current point appended only when newer than last chart point`() {
        val lastChartTimestamp = 1000
        val model = buildChartUIModel(
            prices = listOf(ChartValue(timestamp = lastChartTimestamp, value = 100.0f)),
            priceInfo = createPriceInfo(updatedAt = lastChartTimestamp * 1000L + 1),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertNotNull(model.currentPoint)
        assertEquals(2, model.chartPoints.size)

        val staleModel = buildChartUIModel(
            prices = listOf(ChartValue(timestamp = lastChartTimestamp, value = 100.0f)),
            priceInfo = createPriceInfo(updatedAt = lastChartTimestamp * 1000L - 1),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertNull(staleModel.currentPoint)
        assertEquals(1, staleModel.chartPoints.size)
    }

    @Test
    fun `day period uses 24h change for current point`() {
        val dayModel = buildChartUIModel(
            prices = listOf(ChartValue(timestamp = 1, value = 100.0f)),
            priceInfo = createPriceInfo(price = 200.0, priceChangePercentage24h = 4.2, updatedAt = 2000L),
            period = ChartPeriod.Day,
            currency = Currency.USD,
        )
        assertTrue(dayModel.currentPoint!!.percentage.contains("4.2"))

        val weekModel = buildChartUIModel(
            prices = listOf(ChartValue(timestamp = 1, value = 100.0f)),
            priceInfo = createPriceInfo(price = 200.0, priceChangePercentage24h = 4.2, updatedAt = 2000L),
            period = ChartPeriod.Week,
            currency = Currency.USD,
        )
        // (200 - 100) / 100 * 100 = 100%, not 4.2%
        assertTrue(weekModel.currentPoint!!.percentage.contains("100"))
    }

    @Test
    fun `zero start price does not crash`() {
        val model = buildChartUIModel(
            prices = listOf(ChartValue(timestamp = 1, value = 0.0f)),
            priceInfo = createPriceInfo(price = 50.0, updatedAt = 2000L),
            period = ChartPeriod.Week,
            currency = Currency.USD,
        )
        assertNotNull(model.currentPoint)
        assertTrue(model.currentPoint!!.percentage.contains("0"))
    }

    private fun createPriceInfo(
        price: Double = 123.45,
        priceChangePercentage24h: Double = 4.2,
        updatedAt: Long = Long.MAX_VALUE,
    ) = AssetPriceInfo(
        currency = Currency.USD,
        price = AssetPrice(
            assetId = AssetId(chain = Chain.Bitcoin),
            price = price,
            priceChangePercentage24h = priceChangePercentage24h,
            updatedAt = updatedAt,
        ),
    )
}
