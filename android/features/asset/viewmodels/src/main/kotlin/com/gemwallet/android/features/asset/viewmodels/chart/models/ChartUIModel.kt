package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toPriceState
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency

class ChartUIModel(
    val period: ChartPeriod = ChartPeriod.Day,
    val currentPoint: PricePoint? = null,
    val chartPoints: List<PricePoint> = emptyList(),
) {
    val renderPoints: List<ChartPoint> by lazy {
        chartPoints.mapIndexed { index, point -> ChartPoint(x = index.toFloat(), y = point.y) }
    }

    val minLabel: String? by lazy { chartPoints.minByOrNull { it.y }?.yLabel }
    val maxLabel: String? by lazy { chartPoints.maxByOrNull { it.y }?.yLabel }

    companion object

    class State(
        val loading: Boolean = true,
        val period: ChartPeriod = ChartPeriod.Day,
        val empty: Boolean = false,
    )
}

internal fun ChartUIModel.Companion.from(
    prices: List<ChartValue>,
    priceInfo: AssetPriceInfo?,
    period: ChartPeriod,
    currency: Currency,
): ChartUIModel {
    val basePrice = prices.firstOrNull { it.value != 0.0f }?.value ?: 0.0f
    val historicalPoints = prices.map { chartValue ->
        val changePercent = percentageChange(basePrice, chartValue.value.toDouble())
        PricePoint(
            y = chartValue.value,
            yLabel = currency.format(chartValue.value, 2, dynamicPlace = true),
            timestamp = chartValue.timestamp * 1000L,
            percentage = changePercent.formatAsPercentage(),
            priceState = changePercent.toPriceState(),
        )
    }
    val lastTimestampMillis = (prices.lastOrNull()?.timestamp ?: 0) * 1000L
    val currentPoint = priceInfo
        ?.takeIf { historicalPoints.isNotEmpty() && it.price.updatedAt >= lastTimestampMillis }
        ?.let { info ->
            val changePercent = if (period == ChartPeriod.Day) {
                info.price.priceChangePercentage24h
            } else {
                percentageChange(basePrice, info.price.price)
            }
            PricePoint(
                y = info.price.price.toFloat(),
                yLabel = currency.format(info.price.price.toFloat(), 2, dynamicPlace = true),
                timestamp = System.currentTimeMillis(),
                percentage = changePercent.formatAsPercentage(),
                priceState = changePercent.toPriceState(),
            )
        }

    return ChartUIModel(
        period = period,
        currentPoint = currentPoint,
        chartPoints = historicalPoints + listOfNotNull(currentPoint),
    )
}

private fun percentageChange(basePrice: Float, currentPrice: Double): Double {
    if (basePrice == 0.0f) return 0.0
    return (currentPrice - basePrice) / basePrice * 100.0
}
