package com.gemwallet.android.application.perpetual.coordinators

import com.wallet.core.primitives.PerpetualAccountMode

interface SyncPerpetualPositions {
    suspend fun syncPerpetualPositions(): PerpetualAccountMode?
}