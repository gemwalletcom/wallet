package com.gemwallet.android.data.services.gemstone.lightning

import android.util.Log
import com.gemwallet.android.application.lightning.cases.OpenLightning
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import uniffi.gemstone.GemLightningServiceInterface
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletSessionServiceInterface
import uniffi.gemstone.WalletId
import java.util.concurrent.ConcurrentHashMap

class LightningSessionObserver(private val service: GemLightningServiceInterface, private val session: GemWalletSessionServiceInterface, private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)) :
    OpenLightning {
    private val opening = ConcurrentHashMap<WalletId, Any>()

    override fun openLightning() {
        scope.launch {
            runCatchingCancellable { session.getCurrentWalletId()?.let { openEnabled(it) } }
                .onFailure { if (it !is GemServiceException.Cancelled) Log.e(TAG, "lightning open failed", it) }
        }
    }

    fun lock() {
        opening.clear()
        service.lockAll()
    }

    private suspend fun openEnabled(walletId: WalletId) {
        val attempt = Any()
        if (opening.putIfAbsent(walletId, attempt) != null) return
        try {
            service.openEnabled(walletId)
        } finally {
            opening.remove(walletId, attempt)
        }
    }

    private companion object {
        const val TAG = "Lightning"
    }
}
