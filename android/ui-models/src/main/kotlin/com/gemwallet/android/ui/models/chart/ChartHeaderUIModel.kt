package com.gemwallet.android.ui.models.chart

import com.gemwallet.android.model.text
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.GemChartValueType
import uniffi.gemstone.GemValueTone

data class ChartHeaderUIModel(
    val priceText: String,
    val priceTone: GemValueTone,
    val changeText: String?,
    val changeTone: GemValueTone,
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
            priceTone = header.value.tone,
            changeText = header.change?.text(),
            changeTone = header.change?.tone ?: GemValueTone.NEUTRAL,
            dateText = timestamp?.let(dateFormatter),
            headerValueText = header.secondaryValue?.text(),
            type = type,
        )
    }
}
