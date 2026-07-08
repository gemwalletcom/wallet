package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection

enum class ChartValueType { Price, PriceChange }

data class ChartHeaderUIModel(
    val priceText: String,
    val changeText: String?,
    val direction: ValueDirection,
    val dateText: String?,
    val headerValueText: String? = null,
    val type: ChartValueType = ChartValueType.Price,
) {
    companion object {
        fun build(
            price: Double,
            priceChangePercentage: Double,
            type: ChartValueType = ChartValueType.Price,
            timestamp: Long? = null,
            headerValue: Double? = null,
            priceFormatter: (Double) -> String,
            priceChangeFormatter: (Double) -> String = priceFormatter,
            dateFormatter: (Long) -> String = { "" },
        ): ChartHeaderUIModel = ChartHeaderUIModel(
            priceText = when (type) {
                ChartValueType.Price -> priceFormatter(price)
                ChartValueType.PriceChange -> priceChangeFormatter(price)
            },
            changeText = when (type) {
                ChartValueType.Price -> priceChangePercentage.formatAsPercentage()
                ChartValueType.PriceChange ->
                    if (headerValue != null && priceChangePercentage != 0.0) {
                        "(${priceChangePercentage.formatAsPercentage(PercentageFormatterStyle.PercentSignLess)})"
                    } else {
                        null
                    }
            },
            direction = (if (type == ChartValueType.PriceChange) price else priceChangePercentage).toValueDirection(),
            dateText = timestamp?.let(dateFormatter),
            headerValueText = headerValue?.let(priceFormatter),
            type = type,
        )
    }
}
