package com.gemwallet.android.services

import com.gemwallet.android.application.config.coordinators.GetRemoteConfig
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAssetsService
import javax.inject.Inject

class SyncService @Inject constructor(
    private val getRemoteConfig: GetRemoteConfig,
    private val assetsService: GemAssetsService,
    private val syncDevice: SyncDevice,
) {
    suspend fun sync() {
        withContext(Dispatchers.IO) {
            runCatching { assetsService.syncAvailability(getRemoteConfig.getRemoteConfig().versions.toJson()) }
            runCatching { syncDevice.syncDevice() }
        }
    }
}
