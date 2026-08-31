package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.ChartPeriod

interface SetChartPeriod {
    operator fun invoke(period: ChartPeriod)
}
