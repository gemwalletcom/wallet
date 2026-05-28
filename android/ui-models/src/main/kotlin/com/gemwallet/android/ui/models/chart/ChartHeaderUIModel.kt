package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection

data class ChartHeaderUIModel(
    val priceText: String,
    val changeText: String,
    val direction: ValueDirection,
    val dateText: String?,
    val headerValueText: String? = null,
) {
    companion object {
        fun build(
            price: Double,
            priceChangePercentage: Double,
            timestamp: Long? = null,
            headerValue: Double? = null,
            priceFormatter: (Double) -> String,
            dateFormatter: (Long) -> String = { "" },
        ): ChartHeaderUIModel = ChartHeaderUIModel(
            priceText = priceFormatter(price),
            changeText = priceChangePercentage.formatAsPercentage(),
            direction = priceChangePercentage.toValueDirection(),
            dateText = timestamp?.let(dateFormatter),
            headerValueText = headerValue?.let(priceFormatter),
        )
    }
}
