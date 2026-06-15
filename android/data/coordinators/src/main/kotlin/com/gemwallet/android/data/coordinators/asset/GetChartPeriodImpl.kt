package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetChartPeriod
import com.gemwallet.android.data.repositories.config.UserConfig
import com.wallet.core.primitives.ChartPeriod

class GetChartPeriodImpl(
    private val userConfig: UserConfig,
) : GetChartPeriod {

    override fun invoke(): ChartPeriod {
        return userConfig.chartPeriod()
    }
}
