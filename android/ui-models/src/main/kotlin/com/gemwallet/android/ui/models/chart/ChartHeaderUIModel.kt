package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.GemChartValueType

data class ChartHeaderUIModel(
    val priceText: String,
    val changeText: String?,
    val direction: ValueDirection,
    val dateText: String?,
    val headerValueText: String? = null,
    val type: GemChartValueType = GemChartValueType.PRICE,
) {
    companion object {
        fun build(
            header: GemChartHeader,
            type: GemChartValueType = GemChartValueType.PRICE,
            timestamp: Long? = null,
            priceFormatter: (Double) -> String,
            priceChangeFormatter: (Double) -> String = priceFormatter,
            dateFormatter: (Long) -> String = { "" },
        ): ChartHeaderUIModel = ChartHeaderUIModel(
            priceText = when (type) {
                GemChartValueType.PRICE -> priceFormatter(header.value)
                GemChartValueType.PRICE_CHANGE -> priceChangeFormatter(header.value)
            },
            changeText = header.changePercentage?.let { percentage ->
                when (type) {
                    GemChartValueType.PRICE -> percentage.formatAsPercentage()
                    GemChartValueType.PRICE_CHANGE -> "(${percentage.formatAsPercentage(PercentageFormatterStyle.PercentSignLess)})"
                }
            },
            direction = (if (type == GemChartValueType.PRICE_CHANGE) header.value else header.changePercentage ?: 0.0).toValueDirection(),
            dateText = timestamp?.let(dateFormatter),
            headerValueText = header.secondaryValue?.let(priceFormatter),
            type = type,
        )
    }
}
