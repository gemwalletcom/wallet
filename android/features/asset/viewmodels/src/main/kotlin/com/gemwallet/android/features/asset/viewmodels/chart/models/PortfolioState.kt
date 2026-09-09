package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioType
import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.PortfolioChartType
import uniffi.gemstone.PortfolioData

internal data class PortfolioState(
    val type: PortfolioType,
    val period: ChartPeriod,
    val currency: Currency,
    val data: StateViewType<PortfolioData> = StateViewType.Loading,
)

internal fun PortfolioData.chartValues(chartType: PortfolioChartType): List<ChartDateValue> =
    (charts.firstOrNull { it.chartType == chartType } ?: charts.firstOrNull())?.values.orEmpty()

internal fun List<ChartDateValue>.hasVariation(): Boolean =
    size >= MinChartPoints && minOf { it.value } != maxOf { it.value }
