package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.domains.price.ValueDirection

class PricePoint(
    val y: Float,
    val yLabel: String?,
    val timestamp: Long,
    val percentage: String,
    val priceState: ValueDirection,
)