package com.gemwallet.android.data.repositories.device

import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

internal class DeviceSyncCoordinator {

    private val mutex = Mutex()

    suspend fun synchronize(synchronize: suspend () -> Unit) {
        mutex.withLock {
            synchronize()
        }
    }
}
