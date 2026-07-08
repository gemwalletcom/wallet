package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.domains.price.PriceChange
import com.gemwallet.android.ext.secondsToMillis
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.chart.ChartHeaderUIModel
import com.gemwallet.android.ui.models.chart.ChartValueType
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValue
import com.wallet.core.primitives.Currency

internal const val MinChartPoints = 2
internal const val StopTimeoutMillis = 5_000L

data class ChartUIModel(
    val period: ChartPeriod = ChartPeriod.Day,
    val currentPoint: PricePoint? = null,
    val chartPoints: List<PricePoint> = emptyList(),
    internal val priceFormatter: (Double) -> String = { "" },
    internal val priceChangeFormatter: (Double) -> String = { "" },
    internal val showHeaderValue: Boolean = true,
) {
    val renderPoints: List<ChartPoint> by lazy {
        chartPoints.mapIndexed { index, point -> ChartPoint(x = index.toFloat(), y = point.y) }
    }

    val minLabel: String? by lazy { chartPoints.minByOrNull { it.y }?.price?.let(priceFormatter) }
    val maxLabel: String? by lazy { chartPoints.maxByOrNull { it.y }?.price?.let(priceFormatter) }

    companion object {}

    data class State(
        val period: ChartPeriod = ChartPeriod.Day,
        val chart: StateViewType<ChartUIModel> = StateViewType.Loading,
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

internal fun ChartUIModel.Companion.from(
    values: List<ChartDateValue>,
    period: ChartPeriod,
    currency: Currency,
    showHeaderValue: Boolean,
): ChartUIModel {
    val basePrice = values.firstOrNull()?.value ?: 0.0
    val currencyFormatter = CurrencyFormatter(currency = currency)
    val points = values.map { value ->
        PricePoint(
            y = value.value.toFloat(),
            price = value.value,
            priceChangePercentage = PriceChange.percentage(from = basePrice, to = value.value),
            timestamp = value.date,
        )
    }
    return ChartUIModel(
        period = period,
        chartPoints = points,
        priceFormatter = currencyFormatter::string,
        priceChangeFormatter = PriceChangeFormatter(currencyFormatter)::string,
        showHeaderValue = showHeaderValue,
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

fun portfolioChartHeader(uiModel: ChartUIModel, selectedPoint: PricePoint?): ChartHeaderUIModel? {
    val target = selectedPoint ?: uiModel.chartPoints.lastOrNull() ?: return null
    val base = uiModel.chartPoints.firstOrNull()?.price ?: 0.0
    return ChartHeaderUIModel.build(
        price = target.price - base,
        priceChangePercentage = target.priceChangePercentage,
        type = ChartValueType.PriceChange,
        timestamp = selectedPoint?.timestamp,
        headerValue = if (uiModel.showHeaderValue) target.price else null,
        priceFormatter = uiModel.priceFormatter,
        priceChangeFormatter = uiModel.priceChangeFormatter,
        dateFormatter = ::getRelativeDate,
    )
}
