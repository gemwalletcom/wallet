package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.math.getChartDate
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartBounds
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.chartDateStyle

internal const val StopTimeoutMillis = 5_000L

data class ChartUIModel(val chart: GemChartData) {
    private val priceFormatter by lazy { CurrencyFormatter(currency = chart.currency.toPrimitives()) }

    val renderPoints: List<ChartPoint> by lazy {
        chart.values.mapIndexed { index, value -> ChartPoint(x = index.toFloat(), y = value.value.toFloat()) }
    }

    val bounds: GemChartBounds by lazy { chart.bounds() }

    val minLabel: String? by lazy { chart.values.getOrNull(bounds.lowerIndex.toInt())?.let { priceFormatter.string(it.value) } }
    val maxLabel: String? by lazy { chart.values.getOrNull(bounds.upperIndex.toInt())?.let { priceFormatter.string(it.value) } }

    fun header(selectedIndex: Int?): GemChartHeader? {
        val selected = selectedIndex?.let { chart.values.getOrNull(it) }
        return selected?.let { chart.headerAt(it.value) } ?: chart.header
    }

    fun dateText(selectedIndex: Int?, period: ChartPeriod): String? {
        val selected = selectedIndex?.let { chart.values.getOrNull(it) } ?: return null
        return getChartDate(selected.date, chartDateStyle(period.toGem()))
    }

    data class State(val period: ChartPeriod = ChartPeriod.Day, val chart: StateViewType<ChartUIModel> = StateViewType.Loading)
}
