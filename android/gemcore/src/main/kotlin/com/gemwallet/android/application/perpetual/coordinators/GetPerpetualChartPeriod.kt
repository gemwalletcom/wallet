package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.ChartPeriod

interface GetPerpetualChartPeriod {
    operator fun invoke(): ChartPeriod
}
