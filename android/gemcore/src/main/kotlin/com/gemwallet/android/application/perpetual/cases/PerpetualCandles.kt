package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod

interface PerpetualCandles {
    fun candleInterval(period: ChartPeriod): String

    fun mergeCandle(candles: List<ChartCandleStick>, candle: ChartCandleStick): List<ChartCandleStick>
}
