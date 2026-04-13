package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.application.assets.coordinators.GetHideBalancesState
import com.gemwallet.android.data.repositories.config.UserConfig
import kotlinx.coroutines.flow.Flow

class GetHideBalancesStateImpl(
    private val userConfig: UserConfig,
) : GetHideBalancesState {

    override fun invoke(): Flow<Boolean> {
        return userConfig.isHideBalances()
    }
}
