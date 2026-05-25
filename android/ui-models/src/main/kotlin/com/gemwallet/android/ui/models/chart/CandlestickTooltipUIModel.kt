package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.PriceChange
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.wallet.core.primitives.ChartCandleStick

data class CandlestickTooltipUIModel(
    val open: String,
    val high: String,
    val low: String,
    val close: String,
    val changeText: String,
    val changeDirection: ValueDirection,
    val volumeText: String,
) {
    companion object {
        fun from(
            candle: ChartCandleStick,
            priceFormatter: (Double) -> String,
            volumeFormatter: (Double) -> String,
        ): CandlestickTooltipUIModel {
            val changePercent = PriceChange.percentage(from = candle.open, to = candle.close)
            return CandlestickTooltipUIModel(
                open = priceFormatter(candle.open),
                high = priceFormatter(candle.high),
                low = priceFormatter(candle.low),
                close = priceFormatter(candle.close),
                changeText = changePercent.formatAsPercentage(),
                changeDirection = changePercent.toValueDirection(),
                volumeText = volumeFormatter(candle.volume * candle.close),
            )
        }
    }
}
