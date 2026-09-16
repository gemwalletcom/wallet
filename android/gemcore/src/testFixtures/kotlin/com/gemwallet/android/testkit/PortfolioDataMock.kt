package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.ChartPeriod
import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.PortfolioChartData
import uniffi.gemstone.PortfolioChartType
import uniffi.gemstone.PortfolioData
import uniffi.gemstone.PortfolioStatistic
import java.util.concurrent.TimeUnit

fun mockPortfolioData(
    values: List<Float> = listOf(1f, 2f),
    statistics: List<PortfolioStatistic> = emptyList(),
    availablePeriods: List<ChartPeriod> = emptyList(),
) = PortfolioData(
    charts = listOf(
        PortfolioChartData(
            chartType = PortfolioChartType.VALUE,
            values = values.mapIndexed { index, value ->
                ChartDateValue(date = TimeUnit.SECONDS.toMillis((index + 1).toLong()), value = value.toDouble())
            },
        ),
    ),
    statistics = statistics,
    availablePeriods = availablePeriods.map { it.toGem() },
)
