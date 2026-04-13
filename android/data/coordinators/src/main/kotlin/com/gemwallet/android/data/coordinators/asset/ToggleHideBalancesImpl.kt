package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.ToggleHideBalances
import com.gemwallet.android.data.repositories.config.UserConfig

class ToggleHideBalancesImpl(
    private val userConfig: UserConfig,
) : ToggleHideBalances {

    override suspend fun invoke() {
        userConfig.hideBalances()
    }
}
