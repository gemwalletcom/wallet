package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.ChartPeriod

interface SetPerpetualChartPeriod {
    operator fun invoke(period: ChartPeriod)
}
