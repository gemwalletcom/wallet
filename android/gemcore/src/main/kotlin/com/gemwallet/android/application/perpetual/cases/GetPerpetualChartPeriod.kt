package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.ChartPeriod

interface GetPerpetualChartPeriod {
    operator fun invoke(): ChartPeriod
}
