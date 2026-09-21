package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.getRelativeDate
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.chart.ChartHeaderUIModel
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartBounds
import uniffi.gemstone.GemChartData

internal const val StopTimeoutMillis = 5_000L

data class ChartUIModel(val chart: GemChartData) {
    private val priceFormatter by lazy { CurrencyFormatter(currency = chart.currency.toPrimitives()) }

    val renderPoints: List<ChartPoint> by lazy {
        chart.values.mapIndexed { index, value -> ChartPoint(x = index.toFloat(), y = value.value.toFloat()) }
    }

    val bounds: GemChartBounds by lazy { chart.bounds() }

    val minLabel: String? by lazy { chart.values.getOrNull(bounds.lowerIndex.toInt())?.let { priceFormatter.string(it.value) } }
    val maxLabel: String? by lazy { chart.values.getOrNull(bounds.upperIndex.toInt())?.let { priceFormatter.string(it.value) } }

    fun header(selectedIndex: Int?): ChartHeaderUIModel? {
        val selected = selectedIndex?.let { chart.values.getOrNull(it) }
        val header = selected
            ?.let { chart.headerAt(it.value) }
            ?: chart.header
            ?: return null
        return ChartHeaderUIModel.build(
            header = header,
            type = chart.valueType,
            timestamp = selected?.date,
            dateFormatter = ::getRelativeDate,
        )
    }

    data class State(val period: ChartPeriod = ChartPeriod.Day, val chart: StateViewType<ChartUIModel> = StateViewType.Loading)
}
