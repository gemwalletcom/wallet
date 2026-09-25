package com.gemwallet.android.application.update.cases

import uniffi.gemstone.GemAppUpdateOffer

interface SkipAppUpdate {
    suspend fun skipAppUpdate(update: GemAppUpdateOffer)
}
