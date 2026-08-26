package com.gemwallet.android.services

import com.gemwallet.android.cases.device.SyncDevice
import dagger.Lazy
import uniffi.gemstone.GemWalletRequestPreflight

class DeviceSyncPreflight(
    private val syncDevice: Lazy<SyncDevice>,
) : GemWalletRequestPreflight {
    override suspend fun prepare() = syncDevice.get().syncDevice()
}
