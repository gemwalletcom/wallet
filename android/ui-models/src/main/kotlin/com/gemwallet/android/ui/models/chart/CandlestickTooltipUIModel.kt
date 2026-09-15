package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.wallet.core.primitives.ChartCandleStick
import uniffi.gemstone.candleTooltip

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
        fun from(candle: ChartCandleStick): CandlestickTooltipUIModel {
            val tooltip = candleTooltip(candle.toGem())
            return CandlestickTooltipUIModel(
                open = tooltip.open.text(),
                high = tooltip.high.text(),
                low = tooltip.low.text(),
                close = tooltip.close.text(),
                changeText = tooltip.change.text(),
                changeDirection = tooltip.change.value.toValueDirection(),
                volumeText = tooltip.volume.text(),
            )
        }
    }
}
