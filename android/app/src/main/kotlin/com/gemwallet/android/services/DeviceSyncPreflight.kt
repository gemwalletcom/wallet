package com.gemwallet.android.services

import dagger.Lazy
import uniffi.gemstone.GemWalletRequestPreflight
import uniffi.gemstone.GemDeviceService

class DeviceSyncPreflight(
    private val deviceService: Lazy<GemDeviceService>,
) : GemWalletRequestPreflight {
    override suspend fun prepare() = deviceService.get().synchronizeIfNeeded()
}
