package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.ChartDateValue
import uniffi.gemstone.GemChart

internal data class Chart(
    val values: List<ChartDateValue>,
    val current: ChartDateValue?,
)

internal fun GemChart.toChart(): Chart = Chart(
    values = values.map { it.decodeJson<ChartDateValue>() },
    current = current?.decodeJson<ChartDateValue>(),
)
