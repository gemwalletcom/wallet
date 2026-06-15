package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.SetChartPeriod
import com.gemwallet.android.data.repositories.config.UserConfig
import com.wallet.core.primitives.ChartPeriod

class SetChartPeriodImpl(
    private val userConfig: UserConfig,
) : SetChartPeriod {

    override fun invoke(period: ChartPeriod) {
        userConfig.setChartPeriod(period)
    }
}
