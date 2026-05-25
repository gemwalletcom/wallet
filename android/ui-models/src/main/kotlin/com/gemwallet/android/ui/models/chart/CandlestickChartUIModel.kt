package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.ChartCandleStick
import kotlin.math.max

enum class ChartReferenceLineRole { Entry, Liquidation, StopLoss, TakeProfit }

data class ChartReferenceLineUIModel(
    val price: Double,
    val label: String,
    val role: ChartReferenceLineRole,
    val overlapLevel: Int = 0,
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
    val ySpan: Double get() = (yMax - yMin).coerceAtLeast(MIN_SPAN)

    companion object {
        private const val MIN_SPAN = 1e-9
        private const val RANGE_PADDING_FRACTION = 0.05
        private const val FLOOR_FRACTION = 0.95
        private const val REFERENCE_LINE_VISIBILITY_BUFFER = 0.5
        private const val LABEL_OVERLAP_PRICE_FRACTION = 0.06
        const val DEFAULT_Y_TICK_COUNT = 4
        const val DEFAULT_X_TICK_COUNT = 6
        const val MAX_REFERENCE_LINES = 6

        fun from(
            candles: List<ChartCandleStick>,
            yTickFormatter: (Double) -> String,
            referenceLines: List<ChartReferenceLineUIModel> = emptyList(),
            yTickCount: Int = DEFAULT_Y_TICK_COUNT,
            xTickCount: Int = DEFAULT_X_TICK_COUNT,
        ): CandlestickChartUIModel {
            val candleMin = candles.minOfOrNull { it.low } ?: 0.0
            val candleMax = candles.maxOfOrNull { it.high } ?: 1.0
            val visibleReferences = filterVisibleReferences(referenceLines, candleMin, candleMax)
            val (yMin, yMax) = computePaddedRange(candles, visibleReferences)
            val labeledReferences = assignOverlapLevels(visibleReferences, yMax - yMin)
            return CandlestickChartUIModel(
                candles = candles.map(::candleUIModel),
                yMin = yMin,
                yMax = yMax,
                yTicks = buildYTicks(candleMin, candleMax, yMin, yMax, yTickCount, yTickFormatter),
                xGridlineFractions = buildXGridlineFractions(candles, xTickCount),
                referenceLines = labeledReferences,
                currentPriceLabel = candles.lastOrNull()?.close?.let(yTickFormatter).orEmpty(),
            )
        }

        private fun filterVisibleReferences(
            referenceLines: List<ChartReferenceLineUIModel>,
            candleMin: Double,
            candleMax: Double,
        ): List<ChartReferenceLineUIModel> {
            val buffer = (candleMax - candleMin) * REFERENCE_LINE_VISIBILITY_BUFFER
            val low = candleMin - buffer
            val high = candleMax + buffer
            return referenceLines
                .filter { it.price in low..high }
                .sortedBy { it.price }
                .take(MAX_REFERENCE_LINES)
        }

        private fun assignOverlapLevels(
            references: List<ChartReferenceLineUIModel>,
            ySpan: Double,
        ): List<ChartReferenceLineUIModel> {
            if (references.size < 2) return references
            val threshold = ySpan * LABEL_OVERLAP_PRICE_FRACTION
            var lastLevel = 0
            var lastPrice = references.first().price
            return references.mapIndexed { index, reference ->
                if (index == 0) {
                    reference.copy(overlapLevel = 0)
                } else {
                    val newLevel = if (reference.price - lastPrice < threshold) lastLevel + 1 else 0
                    lastLevel = newLevel
                    lastPrice = reference.price
                    reference.copy(overlapLevel = newLevel)
                }
            }
        }

        private fun computePaddedRange(
            candles: List<ChartCandleStick>,
            referenceLines: List<ChartReferenceLineUIModel>,
        ): Pair<Double, Double> {
            if (candles.isEmpty() && referenceLines.isEmpty()) return 0.0 to 1.0
            val referencePrices = referenceLines.map { it.price }
            val lows = candles.map { it.low } + referencePrices
            val highs = candles.map { it.high } + referencePrices
            val rawMin = lows.min()
            val rawMax = highs.max()
            val span = (rawMax - rawMin).coerceAtLeast(kotlin.math.abs(rawMax) * 0.001 + MIN_SPAN)
            val padding = span * RANGE_PADDING_FRACTION
            val yMin = if (rawMin > 0.0) max(rawMin - padding, rawMin * FLOOR_FRACTION) else rawMin - padding
            return yMin to (rawMax + padding)
        }

        private fun buildYTicks(
            candleMin: Double,
            candleMax: Double,
            yMin: Double,
            yMax: Double,
            tickCount: Int,
            formatter: (Double) -> String,
        ): List<ChartAxisTick> {
            val span = yMax - yMin
            if (span <= 0.0 || tickCount < 2) {
                return listOf(ChartAxisTick(value = candleMin, fraction = 0f, label = formatter(candleMin)))
            }
            if (candleMax <= candleMin) {
                return listOf(ChartAxisTick(value = candleMin, fraction = ((candleMin - yMin) / span).toFloat(), label = formatter(candleMin)))
            }
            val step = (candleMax - candleMin) / (tickCount - 1)
            return (0 until tickCount).map { tick ->
                val value = candleMin + step * tick
                ChartAxisTick(value = value, fraction = ((value - yMin) / span).toFloat(), label = formatter(value))
            }
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
