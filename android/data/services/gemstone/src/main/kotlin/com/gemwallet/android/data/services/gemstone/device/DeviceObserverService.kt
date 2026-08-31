package com.gemwallet.android.data.services.gemstone.device

import com.gemwallet.android.application.wallet.cases.GetWallets
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import uniffi.gemstone.GemDeviceService

class DeviceObserverService(
    private val getWallets: GetWallets,
    private val deviceService: GemDeviceService,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) {
    private var observeJob: Job? = null

    fun start() {
        if (observeJob != null) return

        observeJob = scope.launch {
            getWallets().collectLatest {
                runCatching { deviceService.synchronizeIfNeeded() }
            }
        }
    }

    fun stop() {
        observeJob?.cancel()
        observeJob = null
    }
}
