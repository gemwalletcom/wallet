package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.ChartCandleStick
import uniffi.gemstone.GemPerpetualChartLayout
import uniffi.gemstone.GemPerpetualChartLine

data class ChartReferenceLineUIModel(
    val line: GemPerpetualChartLine,
    val label: String,
)

data class ChartAxisTick(val value: Double, val fraction: Float, val label: String)

data class CandleUIModel(
    val open: Double,
    val high: Double,
    val low: Double,
    val close: Double,
    val direction: ValueDirection,
)

data class CandlestickChartUIModel(
    val candles: List<CandleUIModel>,
    val yMin: Double,
    val yMax: Double,
    val yTicks: List<ChartAxisTick>,
    val xGridlineFractions: List<Float>,
    val referenceLines: List<ChartReferenceLineUIModel>,
    val currentPriceLabel: String,
) {
    val ySpan: Double get() = yMax - yMin

    companion object {
        const val DEFAULT_X_TICK_COUNT = 6

        fun from(
            candles: List<ChartCandleStick>,
            layout: GemPerpetualChartLayout,
            yTickFormatter: (Double) -> String,
            lineLabel: (GemPerpetualChartLine) -> String,
            xTickCount: Int = DEFAULT_X_TICK_COUNT,
        ): CandlestickChartUIModel {
            val span = layout.priceHigh - layout.priceLow
            return CandlestickChartUIModel(
                candles = candles.map(::candleUIModel),
                yMin = layout.priceLow,
                yMax = layout.priceHigh,
                yTicks = layout.ticks.map { value ->
                    ChartAxisTick(value = value, fraction = ((value - layout.priceLow) / span).toFloat(), label = yTickFormatter(value))
                },
                xGridlineFractions = buildXGridlineFractions(candles, xTickCount),
                referenceLines = layout.lines.map { ChartReferenceLineUIModel(it, lineLabel(it)) },
                currentPriceLabel = candles.lastOrNull()?.close?.let(yTickFormatter).orEmpty(),
            )
        }

        private fun buildXGridlineFractions(
            candles: List<ChartCandleStick>,
            tickCount: Int,
        ): List<Float> {
            if (candles.size < 2) return emptyList()
            val capped = tickCount.coerceAtMost(candles.size)
            return (0 until capped).map { tick -> tick.toFloat() / (capped - 1) }
        }

        private fun candleUIModel(candle: ChartCandleStick): CandleUIModel = CandleUIModel(
            open = candle.open,
            high = candle.high,
            low = candle.low,
            close = candle.close,
            direction = when {
                candle.close > candle.open -> ValueDirection.Up
                candle.close < candle.open -> ValueDirection.Down
                else -> ValueDirection.None
            },
        )
    }
}
