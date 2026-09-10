package com.gemwallet.android.features.asset.viewmodels.chart.models

import uniffi.gemstone.GemChart
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency

internal data class AssetChartState(
    val period: ChartPeriod,
    val currency: Currency,
    val prices: StateViewType<GemChart> = StateViewType.Loading,
)
