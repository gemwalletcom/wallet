package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.ChartCandleStick
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualChartLayout
import uniffi.gemstone.GemPerpetualChartLine
import uniffi.gemstone.GemPerpetualChartLineKind

class CandlestickChartUIModelTest {

    private val candles = listOf(
        ChartCandleStick(date = 1_000L, open = 10.0, high = 12.0, low = 9.0, close = 11.0, volume = 100.0),
        ChartCandleStick(date = 2_000L, open = 11.0, high = 13.0, low = 10.0, close = 10.0, volume = 110.0),
        ChartCandleStick(date = 3_000L, open = 10.0, high = 10.0, low = 10.0, close = 10.0, volume = 120.0),
    )

    private val layout = GemPerpetualChartLayout(
        priceLow = 8.0,
        priceHigh = 14.0,
        ticks = listOf(9.0, 11.0, 13.0),
        lines = listOf(GemPerpetualChartLine(GemPerpetualChartLineKind.ENTRY, 10.5, 0u)),
    )

    @Test
    fun candleDirectionsReflectOpenVsClose() {
        val model = model()

        assertEquals(ValueDirection.Up, model.candles[0].direction)
        assertEquals(ValueDirection.Down, model.candles[1].direction)
        assertEquals(ValueDirection.None, model.candles[2].direction)
    }

    @Test
    fun yTicksAreFractionsOfTheLayoutRange() {
        val model = model()

        assertEquals(listOf("9.0", "11.0", "13.0"), model.yTicks.map { it.label })
        assertEquals(listOf(1f / 6f, 0.5f, 5f / 6f), model.yTicks.map { it.fraction })
        assertEquals(6.0, model.ySpan, 1e-9)
    }

    @Test
    fun referenceLinesCarryTheirLabels() {
        val model = model()

        assertEquals(listOf("Entry | 10.5"), model.referenceLines.map { it.label })
        assertEquals(GemPerpetualChartLineKind.ENTRY, model.referenceLines.single().line.kind)
    }

    @Test
    fun xGridlineFractionsSpanZeroToOne() {
        val model = model(xTickCount = 2)

        assertEquals(listOf(0f, 1f), model.xGridlineFractions)
    }

    private fun model(xTickCount: Int = CandlestickChartUIModel.DEFAULT_X_TICK_COUNT) = CandlestickChartUIModel.from(
        candles = candles,
        layout = layout,
        yTickFormatter = { "$it" },
        lineLabel = { "Entry | ${it.price}" },
        xTickCount = xTickCount,
    )
}
