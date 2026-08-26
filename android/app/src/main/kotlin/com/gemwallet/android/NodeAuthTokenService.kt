package com.gemwallet.android

import com.gemwallet.android.cases.device.IsDeviceRegistered
import com.gemwallet.android.ext.runCatchingCancellable
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import uniffi.gemstone.GemNodeAuthConfig
import uniffi.gemstone.GemPreferencesStore
import uniffi.gemstone.nodeAuthConfig
import uniffi.gemstone.GemDeviceService
import com.gemwallet.android.serializer.decodeJson

class NodeAuthTokenService(
    private val deviceService: GemDeviceService,
    private val isDeviceRegistered: IsDeviceRegistered,
    private val preferences: GemPreferencesStore,
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
    private val currentTimeSeconds: () -> ULong = { (System.currentTimeMillis() / 1_000).toULong() },
    private val config: GemNodeAuthConfig = nodeAuthConfig(),
) {
    private var updateJob: Job? = null

    fun start() {
        if (updateJob != null) return
        updateJob = scope.launch {
            runCatchingCancellable { updateIfNeeded() }
            while (isActive) {
                delay(config.checkIntervalSeconds.toLong() * 1_000)
                runCatchingCancellable { updateIfNeeded() }
            }
        }
    }

    fun stop() {
        updateJob?.cancel()
        updateJob = null
    }

    internal suspend fun updateIfNeeded() {
        if (!isDeviceRegistered.isDeviceRegistered()) return
        val token = preferences.authToken()
        val now = currentTimeSeconds()
        val remainingTime = token?.expiresAt?.let { expiry ->
            if (expiry > now) expiry - now else 0uL
        } ?: 0uL
        if (remainingTime >= config.refreshThresholdSeconds.toULong()) return
        preferences.setAuthToken(deviceService.getToken().decodeJson())
    }
}
