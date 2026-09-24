package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.format.SectionDateFormatter
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.GemChartBounds
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.chartDateStyle
import java.time.ZoneId
import java.util.Locale

internal const val StopTimeoutMillis = 5_000L

data class ChartUIModel(val chart: GemChartData) {
    val renderPoints: List<ChartPoint> by lazy {
        chart.values.mapIndexed { index, value -> ChartPoint(x = index.toFloat(), y = value.value.toFloat()) }
    }

    val bounds: GemChartBounds by lazy { chart.bounds() }

    fun header(selectedIndex: Int?): GemChartHeader? {
        val selected = selectedIndex?.let { chart.values.getOrNull(it) }
        return selected?.let { chart.headerAt(it.value) } ?: chart.header
    }

    fun dateText(selectedIndex: Int?, period: ChartPeriod, formatter: SectionDateFormatter): String? {
        val selected = selectedIndex?.let { chart.values.getOrNull(it) } ?: return null
        return formatter.chartDate(selected.date, chartDateStyle(period.toGem()), ZoneId.systemDefault(), Locale.getDefault())
    }

    data class State(val period: ChartPeriod = ChartPeriod.Day, val chart: StateViewType<ChartUIModel> = StateViewType.Loading)
}
