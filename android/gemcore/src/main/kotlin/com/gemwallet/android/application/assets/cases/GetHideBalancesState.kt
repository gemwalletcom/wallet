package com.gemwallet.android.application.assets.cases

import kotlinx.coroutines.flow.Flow

interface GetHideBalancesState {
    operator fun invoke(): Flow<Boolean>
}
