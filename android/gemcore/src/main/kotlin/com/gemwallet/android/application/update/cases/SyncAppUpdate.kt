package com.gemwallet.android.application.update.cases

import uniffi.gemstone.GemAppUpdateOffer

interface SyncAppUpdate {
    suspend fun syncAppUpdate(): GemAppUpdateOffer?
}
