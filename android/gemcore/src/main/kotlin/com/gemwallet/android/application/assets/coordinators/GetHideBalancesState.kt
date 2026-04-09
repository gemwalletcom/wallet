package com.gemwallet.android.application.assets.coordinators

import kotlinx.coroutines.flow.Flow

interface GetHideBalancesState {
    operator fun invoke(): Flow<Boolean>
}
