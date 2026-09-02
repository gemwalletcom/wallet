package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemAssetDiscoveryService
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

@Singleton
class DeviceAssetsSyncService @Inject constructor(
    private val deviceService: GemDeviceService,
    private val discoveryService: GemAssetDiscoveryService,
) {
    suspend fun sync(walletId: String) = withContext(Dispatchers.IO) {
        deviceService.synchronizeIfNeeded()
        discoveryService.discover(walletId)
    }
}
