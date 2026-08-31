package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.PerpetualBalance
import kotlinx.coroutines.flow.Flow
import com.gemwallet.android.domains.perpetual.values.PerpetualBalance as PerpetualBalanceDisplay

interface GetPerpetualBalance {
    fun getBalance(): Flow<PerpetualBalance?>

    fun getDisplayBalance(): Flow<PerpetualBalanceDisplay>

    fun getCollateralIncludedInTotal(): Flow<PerpetualBalance?>
}
