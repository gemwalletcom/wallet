package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioData
import com.wallet.core.primitives.PortfolioType

val walletChartPeriods = listOf(
    ChartPeriod.Day,
    ChartPeriod.Week,
    ChartPeriod.Month,
    ChartPeriod.Year,
    ChartPeriod.All,
)

interface GetPortfolioData {
    suspend fun getPortfolioData(
        type: PortfolioType,
        period: ChartPeriod,
        currency: Currency,
    ): PortfolioData
}
