package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.SetPerpetualChartPeriod
import com.gemwallet.android.data.repositories.config.UserConfig
import com.wallet.core.primitives.ChartPeriod

class SetPerpetualChartPeriodImpl(
    private val userConfig: UserConfig,
) : SetPerpetualChartPeriod {

    override fun invoke(period: ChartPeriod) {
        userConfig.setPerpetualChartPeriod(period)
    }
}
