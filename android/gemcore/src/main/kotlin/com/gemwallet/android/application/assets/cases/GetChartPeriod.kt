package com.gemwallet.android.application.assets.cases

import com.wallet.core.primitives.ChartPeriod

interface GetChartPeriod {
    operator fun invoke(): ChartPeriod
}
