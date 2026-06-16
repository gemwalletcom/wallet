package com.gemwallet.android.features.perpetual.viewmodels

import com.wallet.core.primitives.ChartCandleStick
import com.wallet.core.primitives.ChartPeriod

internal val ChartPeriod.hyperliquidInterval: String
    get() = when (this) {
        ChartPeriod.Hour -> "1m"
        ChartPeriod.Day -> "30m"
        ChartPeriod.Week -> "4h"
        ChartPeriod.Month -> "12h"
        ChartPeriod.Year -> "1w"
        ChartPeriod.All -> "1M"
    }

internal fun List<ChartCandleStick>.mergeCandle(candle: ChartCandleStick): List<ChartCandleStick> {
    val last = lastOrNull() ?: return this
    return when {
        last.date == candle.date -> dropLast(1) + candle
        candle.date > last.date -> drop(1) + candle
        else -> this
    }
}
