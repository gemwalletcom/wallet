package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemAssetDiscoveryService
import javax.inject.Inject
import javax.inject.Singleton
import uniffi.gemstone.GemDeviceService

@Singleton
class DeviceAssetsSyncService @Inject constructor(
    private val deviceService: GemDeviceService,
    private val discoveryService: GemAssetDiscoveryService,
    private val sessionRepository: SessionRepository,
) {
    suspend fun sync(walletId: String) {
        deviceService.synchronizeIfNeeded()
        discoveryService.discover(walletId)
    }
}
