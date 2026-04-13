package com.gemwallet.android.services

import com.gemwallet.android.application.fiat.coordinators.SyncFiatAssets
import com.gemwallet.android.cases.device.SyncDeviceInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject

class SyncService @Inject constructor(
    private val syncFiatAssets: SyncFiatAssets,
    private val syncDeviceInfo: SyncDeviceInfo,
) {

    suspend fun sync() = withContext(Dispatchers.IO) {
        syncFiatAssets()
        syncDeviceInfo.syncDeviceInfo()
    }
}
