package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualChartPeriod
import com.gemwallet.android.data.repositories.config.UserConfig
import com.wallet.core.primitives.ChartPeriod

class GetPerpetualChartPeriodImpl(
    private val userConfig: UserConfig,
) : GetPerpetualChartPeriod {

    override fun invoke(): ChartPeriod {
        return userConfig.perpetualChartPeriod()
    }
}
