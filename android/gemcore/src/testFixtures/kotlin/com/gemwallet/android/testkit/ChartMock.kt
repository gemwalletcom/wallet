package com.gemwallet.android.testkit

import com.wallet.core.primitives.ChartValue

fun mockChartPrices(
    startTimestamp: Int = 1000,
    intervalSeconds: Int = 60,
    values: List<Float> = listOf(100f, 105f, 102f, 108f, 110f),
) = values.mapIndexed { index, value ->
    ChartValue(
        timestamp = startTimestamp + index * intervalSeconds,
        value = value,
    )
}
