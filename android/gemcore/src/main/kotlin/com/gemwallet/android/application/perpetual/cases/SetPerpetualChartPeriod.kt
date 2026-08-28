package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.ChartPeriod

interface SetPerpetualChartPeriod {
    operator fun invoke(period: ChartPeriod)
}
