package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.text
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
            dateFormatter: (Long) -> String = { "" },
        ): ChartHeaderUIModel = ChartHeaderUIModel(
            priceText = header.value.text(),
            changeText = header.change?.text(),
            direction = (if (type == GemChartValueType.PRICE_CHANGE) header.value.value else header.change?.value ?: 0.0).toValueDirection(),
            dateText = timestamp?.let(dateFormatter),
            headerValueText = header.secondaryValue?.text(),
            type = type,
        )
    }
}
