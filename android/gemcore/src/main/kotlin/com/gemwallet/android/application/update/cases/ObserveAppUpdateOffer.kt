package com.gemwallet.android.application.update.cases

import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemAppUpdateOffer

interface ObserveAppUpdateOffer {
    fun observeAppUpdateOffer(): Flow<GemAppUpdateOffer?>
}
