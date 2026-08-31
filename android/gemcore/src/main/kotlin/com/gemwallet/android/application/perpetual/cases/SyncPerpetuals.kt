package com.gemwallet.android.application.perpetual.cases

import uniffi.gemstone.GemMarketsRefreshTrigger

interface SyncPerpetuals {
    suspend fun syncPerpetuals(trigger: GemMarketsRefreshTrigger)
}
