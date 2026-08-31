package com.gemwallet.android.application.perpetual.cases

import com.wallet.core.primitives.PerpetualAccountMode

interface SyncPerpetualPositions {
    suspend fun syncPerpetualPositions(): PerpetualAccountMode?
}