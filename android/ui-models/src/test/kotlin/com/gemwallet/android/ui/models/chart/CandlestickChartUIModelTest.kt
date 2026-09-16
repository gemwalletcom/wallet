package com.gemwallet.android.ui.models.chart

import uniffi.gemstone.GemValueTone
import com.gemwallet.android.model.text
import com.wallet.core.primitives.ChartCandleStick
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualChartLayout
import uniffi.gemstone.GemPerpetualChartLine
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.formattedAdaptive

class CandlestickChartUIModelTest {

    private val candles = listOf(
        ChartCandleStick(date = 1_000L, open = 10.0, high = 12.0, low = 9.0, close = 11.0, volume = 100.0),
        ChartCandleStick(date = 2_000L, open = 11.0, high = 13.0, low = 10.0, close = 10.0, volume = 110.0),
        ChartCandleStick(date = 3_000L, open = 10.0, high = 10.0, low = 10.0, close = 10.0, volume = 120.0),
    )

    private val layout = GemPerpetualChartLayout(
        priceLow = 8.0,
        priceHigh = 14.0,
        ticks = listOf(9.0, 11.0, 13.0).map { formattedAdaptive(it, null) },
        xTickCount = 6u,
        lines = listOf(GemPerpetualChartLine(GemPerpetualChartLineKind.ENTRY, formattedAdaptive(10.5, null), 0u)),
        currentPrice = formattedAdaptive(10.0, null),
    )

    @Test
    fun candleDirectionsReflectOpenVsClose() {
        val model = model()

        assertEquals(GemValueTone.POSITIVE, model.candles[0].direction)
        assertEquals(GemValueTone.NEGATIVE, model.candles[1].direction)
        assertEquals(GemValueTone.NEUTRAL, model.candles[2].direction)
    }

    @Test
    fun yTicksAreFractionsOfTheLayoutRange() {
        val model = model()

        assertEquals(listOf("9.00", "11.00", "13.00"), model.yTicks.map { it.label })
        assertEquals(listOf(1f / 6f, 0.5f, 5f / 6f), model.yTicks.map { it.fraction })
        assertEquals(6.0, model.ySpan, 1e-9)
    }

    @Test
    fun referenceLinesCarryTheirLabels() {
        val model = model()

        assertEquals(listOf("Entry | 10.50"), model.referenceLines.map { it.label })
        assertEquals(GemPerpetualChartLineKind.ENTRY, model.referenceLines.single().line.kind)
    }

    @Test
    fun xGridlineFractionsSpanZeroToOne() {
        val model = model(layout.copy(xTickCount = 2u))

        assertEquals(listOf(0f, 1f), model.xGridlineFractions)
    }

    private fun model(layout: GemPerpetualChartLayout = this.layout) = CandlestickChartUIModel.from(
        candles = candles,
        layout = layout,
        lineLabel = { "Entry | ${it.price.text()}" },
    )
}
