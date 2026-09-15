package com.gemwallet.android.testkit

import com.wallet.core.primitives.ChartValue
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.GemChartValueType

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

fun mockChartHeader(
    value: Double = 110.0,
    base: Double = 100.0,
    showsSecondaryValue: Boolean = false,
    valueType: GemChartValueType = GemChartValueType.PRICE,
    currency: Currency = Currency.USD,
): GemChartHeader = GemChartData(
    valueType = valueType,
    base = base,
    showsSecondaryValue = showsSecondaryValue,
    currency = currency.toGem(),
    values = emptyList(),
    header = null,
).headerAt(value)
