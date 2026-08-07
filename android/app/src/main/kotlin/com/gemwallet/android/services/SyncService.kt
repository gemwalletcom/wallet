package com.gemwallet.android.services

import com.gemwallet.android.application.fiat.coordinators.SyncFiatAssets
import com.gemwallet.android.application.swap.coordinators.SyncSwapAssets
import com.gemwallet.android.cases.device.SyncDevice
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class SyncService @Inject constructor(
    private val syncFiatAssets: SyncFiatAssets,
    private val syncSwapAssets: SyncSwapAssets,
    private val syncDevice: SyncDevice,
) {

    suspend fun sync() {
        withContext(Dispatchers.IO) {
            runCatching { syncFiatAssets() }
            runCatching { syncSwapAssets() }
            runCatching { syncDevice.syncDevice() }
        }
    }
}
