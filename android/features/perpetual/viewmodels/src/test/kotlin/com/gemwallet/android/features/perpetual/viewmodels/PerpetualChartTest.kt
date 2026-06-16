package com.gemwallet.android.features.perpetual.viewmodels

import com.wallet.core.primitives.ChartCandleStick
import org.junit.Assert.assertEquals
import org.junit.Test

class PerpetualChartTest {

    private fun candle(date: Long, close: Double = 100.0) = ChartCandleStick(
        date = date,
        open = close - 1,
        high = close + 1,
        low = close - 2,
        close = close,
        volume = 1000.0,
    )

    @Test
    fun `mergeCandle replaces last candle with same date`() {
        val candles = listOf(candle(1000), candle(2000, close = 100.0))
        val merged = candles.mergeCandle(candle(2000, close = 105.0))

        assertEquals(2, merged.size)
        assertEquals(105.0, merged.last().close, 0.0)
        assertEquals(1000L, merged.first().date)
    }

    @Test
    fun `mergeCandle appends newer candle and drops oldest`() {
        val candles = listOf(candle(1000), candle(2000))
        val merged = candles.mergeCandle(candle(3000, close = 110.0))

        assertEquals(2, merged.size)
        assertEquals(2000L, merged.first().date)
        assertEquals(3000L, merged.last().date)
    }

    @Test
    fun `mergeCandle ignores stale and keeps empty list`() {
        val candles = listOf(candle(1000), candle(2000))
        assertEquals(candles, candles.mergeCandle(candle(500)))
        assertEquals(emptyList<ChartCandleStick>(), emptyList<ChartCandleStick>().mergeCandle(candle(500)))
    }
}
