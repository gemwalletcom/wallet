package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.PerpetualBalance
import kotlinx.coroutines.flow.Flow

interface GetPerpetualBalance {
    fun getBalance(): Flow<PerpetualBalance?>
}
