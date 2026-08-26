package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.cases.device.SyncDevice
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemAssetDiscoveryService

@Singleton
class DeviceAssetsSyncService @Inject constructor(
    private val syncDevice: SyncDevice,
    private val discoveryService: GemAssetDiscoveryService,
) {

    suspend fun sync(walletId: String) {
        syncDevice.syncDevice()
        discoveryService.discover(walletId)
    }
}
