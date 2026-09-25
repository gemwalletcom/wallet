package com.gemwallet.android.testkit

import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.GemChart

fun mockGemChart(values: List<Float> = listOf(100f, 105f, 102f, 108f, 110f)) = GemChart(
    values = values.mapIndexed { index, value -> ChartDateValue(date = 1_000L + index * 60_000L, value = value.toDouble()) },
    baseValue = values.firstOrNull()?.toDouble() ?: 0.0,
    current = null,
)
