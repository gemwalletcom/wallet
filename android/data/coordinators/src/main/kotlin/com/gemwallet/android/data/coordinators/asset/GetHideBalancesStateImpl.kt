package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.cases.GetHideBalancesState
import com.gemwallet.android.data.adapters.config.UserConfig
import kotlinx.coroutines.flow.Flow

class GetHideBalancesStateImpl(
    private val userConfig: UserConfig,
) : GetHideBalancesState {

    override fun invoke(): Flow<Boolean> {
        return userConfig.isHideBalances()
    }
}
