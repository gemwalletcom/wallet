package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.ChartCandleStick
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class CandlestickChartUIModelTest {

    private val candles = listOf(
        ChartCandleStick(date = 1_000L, open = 10.0, high = 12.0, low = 9.0, close = 11.0, volume = 100.0),
        ChartCandleStick(date = 2_000L, open = 11.0, high = 13.0, low = 10.0, close = 10.0, volume = 110.0),
        ChartCandleStick(date = 3_000L, open = 10.0, high = 10.0, low = 10.0, close = 10.0, volume = 120.0),
    )

    @Test
    fun candleDirectionsReflectOpenVsClose() {
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "$it" },
        )
        assertEquals(ValueDirection.Up, model.candles[0].direction)
        assertEquals(ValueDirection.Down, model.candles[1].direction)
        assertEquals(ValueDirection.None, model.candles[2].direction)
    }

    @Test
    fun yRangeIncludesInRangeReferencesAndExpandsAccordingly() {
        val referenceLines = listOf(
            ChartReferenceLineUIModel(price = 14.0, label = "TP", role = ChartReferenceLineRole.TakeProfit),
            ChartReferenceLineUIModel(price = 8.0, label = "SL", role = ChartReferenceLineRole.StopLoss),
        )
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "" },
            referenceLines = referenceLines,
        )
        assertEquals(2, model.referenceLines.size)
        assertTrue("yMin (${model.yMin}) must be at or below 8.0", model.yMin <= 8.0)
        assertTrue("yMax (${model.yMax}) must be at or above 14.0", model.yMax >= 14.0)
    }

    @Test
    fun referenceLinesFarOutsideCandleRangeAreFiltered() {
        val referenceLines = listOf(
            ChartReferenceLineUIModel(price = 100.0, label = "TP", role = ChartReferenceLineRole.TakeProfit),
            ChartReferenceLineUIModel(price = 1.0, label = "SL", role = ChartReferenceLineRole.StopLoss),
            ChartReferenceLineUIModel(price = 10.5, label = "Entry", role = ChartReferenceLineRole.Entry),
        )
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "" },
            referenceLines = referenceLines,
        )
        assertEquals(listOf(10.5), model.referenceLines.map { it.price })
    }

    @Test
    fun closeReferencesGetSequentialOverlapLevels() {
        val references = listOf(
            ChartReferenceLineUIModel(price = 10.0, label = "A", role = ChartReferenceLineRole.Entry),
            ChartReferenceLineUIModel(price = 10.1, label = "B", role = ChartReferenceLineRole.StopLoss),
            ChartReferenceLineUIModel(price = 12.5, label = "C", role = ChartReferenceLineRole.TakeProfit),
        )
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "" },
            referenceLines = references,
        )
        val levels = model.referenceLines.associate { it.label to it.overlapLevel }
        assertEquals(0, levels["A"])
        assertEquals(1, levels["B"])
        assertEquals(0, levels["C"])
    }

    @Test
    fun visibleReferencesAreSortedByPriceAscending() {
        val referenceLines = listOf(
            ChartReferenceLineUIModel(price = 12.0, label = "B", role = ChartReferenceLineRole.TakeProfit),
            ChartReferenceLineUIModel(price = 9.5, label = "A", role = ChartReferenceLineRole.StopLoss),
            ChartReferenceLineUIModel(price = 11.0, label = "C", role = ChartReferenceLineRole.Entry),
        )
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "" },
            referenceLines = referenceLines,
        )
        assertEquals(listOf(9.5, 11.0, 12.0), model.referenceLines.map { it.price })
    }

    @Test
    fun yTickCountMatchesIosFour() {
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "%.1f".format(it) },
        )
        assertEquals(4, model.yTicks.size)
        assertEquals(9.0, model.yTicks.first().value, 1e-9)
        assertEquals(13.0, model.yTicks.last().value, 1e-9)
    }

    @Test
    fun xGridlineFractionsSpanZeroToOne() {
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "" },
            xTickCount = 2,
        )
        assertEquals(listOf(0f, 1f), model.xGridlineFractions)
    }

    @Test
    fun positiveLowestRangeRespectsFloorGuard() {
        val model = CandlestickChartUIModel.from(
            candles = candles,
            yTickFormatter = { "" },
        )
        assertTrue("yMin (${model.yMin}) must be at least 95% of candleMin", model.yMin >= 9.0 * 0.95)
    }
}
