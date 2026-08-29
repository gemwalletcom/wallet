package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.SetChartPeriod
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.wallet.core.primitives.ChartPeriod

class SetChartPeriodImpl(
    private val userConfig: UserConfig,
) : SetChartPeriod {

    override fun invoke(period: ChartPeriod) {
        userConfig.setChartPeriod(period)
    }
}
