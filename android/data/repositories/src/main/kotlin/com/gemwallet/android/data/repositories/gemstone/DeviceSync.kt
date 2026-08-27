package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.cases.device.SyncDevice
import uniffi.gemstone.GemDeviceSync

class GemstoneDeviceSync(
    private val syncDevice: SyncDevice,
) : GemDeviceSync {
    override suspend fun syncDevice() = syncDevice.syncDevice()
}
