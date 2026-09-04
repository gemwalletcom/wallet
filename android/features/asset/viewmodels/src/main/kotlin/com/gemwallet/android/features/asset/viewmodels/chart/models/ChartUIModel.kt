package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.domains.price.PriceChangeCalculator
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
    chart: Chart,
    priceInfo: AssetPriceInfo?,
    period: ChartPeriod,
    currency: Currency,
): ChartUIModel {
    val basePrice = chart.values.firstOrNull { it.value != 0.0 }?.value ?: 0.0
    val currencyFormatter = CurrencyFormatter(currency = currency)
    val priceFormatter: (Double) -> String = currencyFormatter::string
    val historicalPoints = chart.values.map { value ->
        PricePoint(
            y = value.value.toFloat(),
            price = value.value,
            priceChangePercentage = PriceChangeCalculator.percentage(from = basePrice, to = value.value),
            timestamp = value.date,
        )
    }
    val currentPoint = chart.current?.let { current ->
        val changePercent = when {
            period == ChartPeriod.Day && priceInfo != null -> priceInfo.price.priceChangePercentage24h
            else -> PriceChangeCalculator.percentage(from = basePrice, to = current.value)
        }
        PricePoint(
            y = current.value.toFloat(),
            price = current.value,
            priceChangePercentage = changePercent,
            timestamp = current.date,
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
            priceChangePercentage = PriceChangeCalculator.percentage(from = basePrice, to = value.value),
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
