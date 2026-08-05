package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.PerpetualAccountMode

interface GetPerpetualAccountMode {
    suspend fun getPerpetualAccountMode(address: String): PerpetualAccountMode
}
