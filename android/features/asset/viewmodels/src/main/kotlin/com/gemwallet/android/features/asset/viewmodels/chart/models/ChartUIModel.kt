package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.chart.ChartBoundLabel
import com.gemwallet.android.ui.components.chart.ChartPoint
import com.gemwallet.android.ui.format.SectionDateFormatter
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.GemChartBounds
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.GemChartViewport
import uniffi.gemstone.chartDateStyle
import java.time.ZoneId
import java.util.Locale

internal const val StopTimeoutMillis = 5_000L

data class ChartUIModel(val chart: GemChartData, val viewport: GemChartViewport) {
    private val span = (viewport.end - viewport.start).coerceAtLeast(1L).toFloat()

    val points: List<ChartPoint> by lazy {
        viewport.renderValues.map { value -> ChartPoint(x = fraction(value.date), y = value.value.toFloat()) }
    }

    val selectableFrom: Int by lazy { viewport.renderValues.indexOfFirst { it.date >= viewport.start }.coerceAtLeast(0) }

    val bounds: GemChartBounds get() = viewport.bounds

    val minLabel: ChartBoundLabel? by lazy { label(bounds.lowerIndex.toInt(), bounds.low.text()) }

    val maxLabel: ChartBoundLabel? by lazy { label(bounds.upperIndex.toInt(), bounds.high.text()) }

    fun header(selectedIndex: Int?): GemChartHeader? = selected(selectedIndex)?.let { chart.headerAt(it.value) } ?: chart.header

    fun dateText(selectedIndex: Int?, period: ChartPeriod, formatter: SectionDateFormatter): String? {
        val selected = selected(selectedIndex) ?: return null
        return formatter.chartDate(selected.date, chartDateStyle(period.toGem()), ZoneId.systemDefault(), Locale.getDefault())
    }

    private fun selected(index: Int?): ChartDateValue? = index?.takeIf { it >= selectableFrom }?.let { viewport.renderValues.getOrNull(it) }

    private fun label(index: Int, text: String): ChartBoundLabel? = viewport.values.getOrNull(index)?.let { ChartBoundLabel(x = fraction(it.date), text = text) }

    private fun fraction(date: Long): Float = (date - viewport.start) / span

    data class State(val period: ChartPeriod = ChartPeriod.Day, val chart: StateViewType<ChartUIModel> = StateViewType.Loading)
}
