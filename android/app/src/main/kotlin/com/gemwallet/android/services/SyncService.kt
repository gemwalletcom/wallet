package com.gemwallet.android.services

import android.util.Log
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.serializer.toJson
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAppStartService
import javax.inject.Inject
import com.gemwallet.android.ext.runCatchingCancellable

class SyncService @Inject constructor(
    private val appStartService: GemAppStartService,
    private val syncDevice: SyncDevice,
) {
    suspend fun sync() {
        withContext(Dispatchers.IO) {
            appStartService.run().forEach { failure ->
                Log.e("SyncService", "${failure.step} failed: ${failure.message}")
            }
            runCatchingCancellable { syncDevice.syncDevice() }
                .onFailure { Log.e("SyncService", "device sync failed", it) }
        }
    }
}
