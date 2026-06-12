package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.domains.price.PriceChange
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.models.chart.ChartHeaderUIModel
import com.gemwallet.android.ui.models.chart.ChartViewState
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency

data class ChartUIModel(
    val period: ChartPeriod = ChartPeriod.Day,
    val currentPoint: PricePoint? = null,
    val chartPoints: List<PricePoint> = emptyList(),
    internal val priceFormatter: (Double) -> String = { "" },
) {
    val renderPoints: List<ChartPoint> by lazy {
        chartPoints.mapIndexed { index, point -> ChartPoint(x = index.toFloat(), y = point.y) }
    }

    val minLabel: String? by lazy { chartPoints.minByOrNull { it.y }?.price?.let(priceFormatter) }
    val maxLabel: String? by lazy { chartPoints.maxByOrNull { it.y }?.price?.let(priceFormatter) }

    companion object {}

    data class State(
        val period: ChartPeriod = ChartPeriod.Day,
        val viewState: ChartViewState = ChartViewState.Loading,
    )
}

internal fun ChartUIModel.Companion.from(
    prices: List<ChartValue>,
    priceInfo: AssetPriceInfo?,
    period: ChartPeriod,
    currency: Currency,
): ChartUIModel {
    val basePrice = prices.firstOrNull { it.value != 0.0f }?.value ?: 0.0f
    val currencyFormatter = CurrencyFormatter(currency = currency)
    val priceFormatter: (Double) -> String = currencyFormatter::string
    val historicalPoints = prices.map { chartValue ->
        PricePoint(
            y = chartValue.value,
            price = chartValue.value.toDouble(),
            priceChangePercentage = PriceChange.percentage(from = basePrice.toDouble(), to = chartValue.value.toDouble()),
            timestamp = chartValue.timestamp.toLong().secondsToMillis(),
        )
    }
    val lastTimestampMillis = (prices.lastOrNull()?.timestamp ?: 0).toLong().secondsToMillis()
    val currentPoint: PricePoint? = priceInfo
        ?.takeIf { historicalPoints.isNotEmpty() && it.price.updatedAt >= lastTimestampMillis }
        ?.let { assetInfo ->
            val changePercent = if (period == ChartPeriod.Day) {
                assetInfo.price.priceChangePercentage24h
            } else {
                PriceChange.percentage(from = basePrice.toDouble(), to = assetInfo.price.price)
            }
            PricePoint(
                y = assetInfo.price.price.toFloat(),
                price = assetInfo.price.price,
                priceChangePercentage = changePercent,
                timestamp = System.currentTimeMillis(),
            )
        }

    return ChartUIModel(
        period = period,
        currentPoint = currentPoint,
        chartPoints = historicalPoints + listOfNotNull(currentPoint),
        priceFormatter = priceFormatter,
    )
}

fun chartHeader(uiModel: ChartUIModel, selectedPoint: PricePoint?): ChartHeaderUIModel? {
    val target = selectedPoint ?: uiModel.chartPoints.lastOrNull() ?: return null
    return ChartHeaderUIModel.build(
        price = target.price,
        priceChangePercentage = target.priceChangePercentage,
        timestamp = selectedPoint?.timestamp,
        priceFormatter = uiModel.priceFormatter,
        dateFormatter = ::getRelativeDate,
    )
}
