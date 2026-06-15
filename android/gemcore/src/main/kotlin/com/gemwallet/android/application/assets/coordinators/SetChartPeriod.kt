package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.ChartPeriod

interface SetChartPeriod {
    operator fun invoke(period: ChartPeriod)
}
