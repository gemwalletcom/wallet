package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import uniffi.gemstone.GemCandleTooltip
import uniffi.gemstone.GemCandleTooltipRow
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.candleTooltip
import com.wallet.core.primitives.ChartCandleStick
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.util.Locale

class CandleTooltipTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    @Test
    fun formatsOhlcAndUsesIosCompatibleSignedPercentForUpwardCandle() {
        val candle = ChartCandleStick(
            date = 0L, open = 100.0, high = 110.0, low = 95.0, close = 105.0, volume = 10.0,
        )
        val model = candleTooltip(candle.toGem())
        assertEquals("100.00", model.text(GemCandleTooltipRow.OPEN))
        assertEquals("110.00", model.text(GemCandleTooltipRow.HIGH))
        assertEquals("95.00", model.text(GemCandleTooltipRow.LOW))
        assertEquals("105.00", model.text(GemCandleTooltipRow.CLOSE))
        assertEquals(GemValueTone.POSITIVE, model.tone(GemCandleTooltipRow.CHANGE))
        assertEquals("+5.00%", model.text(GemCandleTooltipRow.CHANGE))
    }

    @Test
    fun downwardCandleProducesNegativePercentAndDownDirection() {
        val candle = ChartCandleStick(
            date = 0L, open = 100.0, high = 100.0, low = 80.0, close = 90.0, volume = 0.0,
        )
        val model = candleTooltip(candle.toGem())
        assertEquals("-10.00%", model.text(GemCandleTooltipRow.CHANGE))
        assertEquals(GemValueTone.NEGATIVE, model.tone(GemCandleTooltipRow.CHANGE))
    }

    @Test
    fun volumeUsesUsdValueNotRawUnits() {
        val candle = ChartCandleStick(
            date = 0L, open = 50.0, high = 55.0, low = 49.0, close = 52.0, volume = 1_000.0,
        )
        val model = candleTooltip(candle.toGem())
        assertEquals("$52,000.00", model.text(GemCandleTooltipRow.VOLUME))
    }

    @Test
    fun zeroOpenSkipsPercentMathAndUsesNeutralDirection() {
        val candle = ChartCandleStick(
            date = 0L, open = 0.0, high = 5.0, low = 0.0, close = 5.0, volume = 1.0,
        )
        val model = candleTooltip(candle.toGem())
        assertEquals(GemValueTone.NEUTRAL, model.tone(GemCandleTooltipRow.CHANGE))
        assertEquals("+0.00%", model.text(GemCandleTooltipRow.CHANGE))
    }
}

private fun GemCandleTooltip.cell(row: GemCandleTooltipRow) = (prices + summary).first { it.row == row }

private fun GemCandleTooltip.text(row: GemCandleTooltipRow) = cell(row).value.text()

private fun GemCandleTooltip.tone(row: GemCandleTooltipRow) = cell(row).value.tone
