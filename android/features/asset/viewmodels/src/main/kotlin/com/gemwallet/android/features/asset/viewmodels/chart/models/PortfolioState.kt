package com.gemwallet.android.features.asset.viewmodels.chart.models

import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioType
import uniffi.gemstone.PortfolioData

internal data class PortfolioState(
    val type: PortfolioType,
    val period: ChartPeriod,
    val currency: Currency,
    val data: StateViewType<PortfolioData> = StateViewType.Loading,
)
