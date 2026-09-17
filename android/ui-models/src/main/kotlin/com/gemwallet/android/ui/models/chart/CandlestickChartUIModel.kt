package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.model.text
import com.wallet.core.primitives.ChartCandleStick
import uniffi.gemstone.GemPerpetualChartLayout
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemValueTone

enum class ChartReferenceLineKind {
    Entry,
    Liquidation,
    StopLoss,
    TakeProfit,
}

enum class CandleDirection {
    Up,
    Down,
    Flat,
}

data class ChartReferenceLineUIModel(
    val kind: ChartReferenceLineKind,
    val price: Double,
    val overlapLevel: Int,
    val label: String,
)

data class ChartAxisTick(val value: Double, val fraction: Float, val label: String)

data class CandleUIModel(
    val open: Double,
    val high: Double,
    val low: Double,
    val close: Double,
    val direction: CandleDirection,
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

        fun from(
            candles: List<ChartCandleStick>,
            layout: GemPerpetualChartLayout,
            lineLabel: (GemPerpetualChartLineKind) -> String,
        ): CandlestickChartUIModel {
            val span = layout.priceHigh - layout.priceLow
            return CandlestickChartUIModel(
                candles = candles.map(::candleUIModel),
                yMin = layout.priceLow,
                yMax = layout.priceHigh,
                yTicks = layout.ticks.map { tick ->
                    ChartAxisTick(value = tick.value, fraction = ((tick.value - layout.priceLow) / span).toFloat(), label = tick.text())
                },
                xGridlineFractions = buildXGridlineFractions(layout.xTickCount.toInt()),
                referenceLines = layout.lines.map { line ->
                    ChartReferenceLineUIModel(
                        kind = line.kind.referenceLineKind(),
                        price = line.price.value,
                        overlapLevel = line.overlapLevel.toInt(),
                        label = "${lineLabel(line.kind)} | ${line.price.text()}",
                    )
                },
                currentPriceLabel = layout.currentPrice?.text().orEmpty(),
            )
        }

        private fun buildXGridlineFractions(tickCount: Int): List<Float> {
            if (tickCount < 2) return emptyList()
            return (0 until tickCount).map { tick -> tick.toFloat() / (tickCount - 1) }
        }

        private fun candleUIModel(candle: ChartCandleStick): CandleUIModel = CandleUIModel(
            open = candle.open,
            high = candle.high,
            low = candle.low,
            close = candle.close,
            direction = when ((candle.close - candle.open).tone()) {
                GemValueTone.POSITIVE -> CandleDirection.Up
                GemValueTone.NEGATIVE -> CandleDirection.Down
                GemValueTone.NEUTRAL,
                GemValueTone.PLAIN -> CandleDirection.Flat
            },
        )

        private fun GemPerpetualChartLineKind.referenceLineKind(): ChartReferenceLineKind = when (this) {
            GemPerpetualChartLineKind.ENTRY -> ChartReferenceLineKind.Entry
            GemPerpetualChartLineKind.LIQUIDATION -> ChartReferenceLineKind.Liquidation
            GemPerpetualChartLineKind.STOP_LOSS -> ChartReferenceLineKind.StopLoss
            GemPerpetualChartLineKind.TAKE_PROFIT -> ChartReferenceLineKind.TakeProfit
        }
    }
}
