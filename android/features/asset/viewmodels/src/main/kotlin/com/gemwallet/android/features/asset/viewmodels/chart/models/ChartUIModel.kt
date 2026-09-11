package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.chart.ChartHeaderUIModel
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartData
import uniffi.gemstone.chartHeader

internal const val StopTimeoutMillis = 5_000L

data class ChartUIModel(
    val chart: GemChartData,
    internal val priceFormatter: (Double) -> String,
    internal val priceChangeFormatter: (Double) -> String = priceFormatter,
) {
    val renderPoints: List<ChartPoint> by lazy {
        chart.values.mapIndexed { index, value -> ChartPoint(x = index.toFloat(), y = value.value.toFloat()) }
    }

    val minLabel: String? by lazy { chart.values.minByOrNull { it.value }?.value?.let(priceFormatter) }
    val maxLabel: String? by lazy { chart.values.maxByOrNull { it.value }?.value?.let(priceFormatter) }

    fun header(selectedIndex: Int?): ChartHeaderUIModel? {
        val selected = selectedIndex?.let { chart.values.getOrNull(it) }
        val header = selected
            ?.let { chartHeader(chart.valueType, chart.base, it.value, chart.showsSecondaryValue) }
            ?: chart.header
            ?: return null
        return ChartHeaderUIModel.build(
            header = header,
            type = chart.valueType,
            timestamp = selected?.date,
            priceFormatter = priceFormatter,
            priceChangeFormatter = priceChangeFormatter,
            dateFormatter = ::getRelativeDate,
        )
    }

    data class State(
        val period: ChartPeriod = ChartPeriod.Day,
        val chart: StateViewType<ChartUIModel> = StateViewType.Loading,
    )
}
