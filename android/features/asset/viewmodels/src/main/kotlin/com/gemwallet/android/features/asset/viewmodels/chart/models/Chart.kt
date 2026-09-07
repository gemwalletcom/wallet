package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.ChartDateValue
import uniffi.gemstone.GemChart

internal data class Chart(
    val values: List<ChartDateValue>,
    val current: ChartDateValue?,
)

internal fun GemChart.toChart(): Chart = Chart(
    values = values.map { it.toPrimitives() },
    current = current?.toPrimitives(),
)
