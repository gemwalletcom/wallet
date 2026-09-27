package com.gemwallet.android.data.services.gemstone.device

import com.gemwallet.android.data.services.store.queries.WalletsQuery
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import uniffi.gemstone.GemDeviceService
import uniffi.gemstone.GemDeviceServiceInterface

class DeviceObserverService(private val walletsQuery: WalletsQuery, private val deviceService: GemDeviceServiceInterface, private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)) {
    private var observeJob: Job? = null

    fun start() {
        if (observeJob != null) return

        observeJob = scope.launch {
            walletsQuery().collectLatest {
                runCatching { deviceService.synchronizeIfNeeded() }
            }
        }
    }

    fun stop() {
        observeJob?.cancel()
        observeJob = null
    }
}
