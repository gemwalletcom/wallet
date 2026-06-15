package com.gemwallet.android.application.assets.coordinators

import com.wallet.core.primitives.ChartPeriod

interface GetChartPeriod {
    operator fun invoke(): ChartPeriod
}
