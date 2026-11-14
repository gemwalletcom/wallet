package com.gemwallet.android.data.services.gemstone.lightning

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemServiceException

class LightningSessionObserver(private val service: GemLightningServiceInterface, private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)) {

    fun openLightning() {
        scope.launch {
            runCatchingCancellable { service.openEnabled() }
                .onFailure { if (it !is GemServiceException.Cancelled) Log.e(TAG, "lightning open failed", it) }
        }
    }

    private companion object {
        const val TAG = "Lightning"
    }
}
