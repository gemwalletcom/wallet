package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.ChartCandleStick
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.util.Locale

class CandlestickTooltipUIModelTest {

    @Before
    fun setUp() {
        Locale.setDefault(Locale.US)
    }

    private val priceFormatter: (Double) -> String = { "%.2f".format(it) }
    private val volumeFormatter: (Double) -> String = { "vol(%.2f)".format(it) }

    @Test
    fun formatsOhlcAndUsesIosCompatibleSignedPercentForUpwardCandle() {
        val candle = ChartCandleStick(
            date = 0L, open = 100.0, high = 110.0, low = 95.0, close = 105.0, volume = 10.0,
        )
        val model = CandlestickTooltipUIModel.from(candle, priceFormatter, volumeFormatter)
        assertEquals("100.00", model.open)
        assertEquals("110.00", model.high)
        assertEquals("95.00", model.low)
        assertEquals("105.00", model.close)
        assertEquals(ValueDirection.Up, model.changeDirection)
        assertEquals("+5.00%", model.changeText)
    }

    @Test
    fun downwardCandleProducesNegativePercentAndDownDirection() {
        val candle = ChartCandleStick(
            date = 0L, open = 100.0, high = 100.0, low = 80.0, close = 90.0, volume = 0.0,
        )
        val model = CandlestickTooltipUIModel.from(candle, priceFormatter, volumeFormatter)
        assertEquals("-10.00%", model.changeText)
        assertEquals(ValueDirection.Down, model.changeDirection)
    }

    @Test
    fun volumeUsesUsdValueNotRawUnits() {
        val candle = ChartCandleStick(
            date = 0L, open = 50.0, high = 55.0, low = 49.0, close = 52.0, volume = 1_000.0,
        )
        val model = CandlestickTooltipUIModel.from(candle, priceFormatter, volumeFormatter)
        assertEquals("vol(52000.00)", model.volumeText)
    }

    @Test
    fun zeroOpenSkipsPercentMathAndUsesNeutralDirection() {
        val candle = ChartCandleStick(
            date = 0L, open = 0.0, high = 5.0, low = 0.0, close = 5.0, volume = 1.0,
        )
        val model = CandlestickTooltipUIModel.from(candle, priceFormatter, volumeFormatter)
        assertEquals(ValueDirection.None, model.changeDirection)
        assertEquals("+0.00%", model.changeText)
    }
}
