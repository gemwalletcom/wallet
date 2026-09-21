package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Currency
import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.GemChart
import uniffi.gemstone.GemChartData
import uniffi.gemstone.GemChartHeader
import uniffi.gemstone.GemChartValueType

fun mockChartHeader(value: Double = 110.0, base: Double = 100.0, showsSecondaryValue: Boolean = false, valueType: GemChartValueType = GemChartValueType.PRICE): GemChartHeader = GemChartData(
    valueType = valueType,
    base = base,
    showsSecondaryValue = showsSecondaryValue,
    currency = Currency.USD.toGem(),
    values = emptyList(),
    header = null,
).headerAt(value)

fun mockGemChart(values: List<Float> = listOf(100f, 105f, 102f, 108f, 110f)) = GemChart(
    values = values.mapIndexed { index, value -> ChartDateValue(date = 1_000L + index * 60_000L, value = value.toDouble()) },
    baseValue = values.firstOrNull()?.toDouble() ?: 0.0,
    current = null,
)
