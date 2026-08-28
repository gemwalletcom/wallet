package com.gemwallet.android.data.repositories.device

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import uniffi.gemstone.GemDeviceService

class DeviceObserverService(
    private val walletsRepository: WalletsRepository,
    private val deviceService: GemDeviceService,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO),
) {
    private var observeJob: Job? = null

    fun start() {
        if (observeJob != null) return

        observeJob = scope.launch {
            walletsRepository.getAll().collectLatest {
                runCatching { deviceService.synchronizeIfNeeded() }
            }
        }
    }

    fun stop() {
        observeJob?.cancel()
        observeJob = null
    }
}
