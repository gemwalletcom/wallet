package com.gemwallet.android

import android.os.SystemClock
import androidx.annotation.VisibleForTesting
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import kotlinx.coroutines.flow.first
import java.util.concurrent.atomic.AtomicLong
import uniffi.gemstone.GemSecurityService
import javax.inject.Inject

class LockTimer @Inject constructor(
    private val userConfig: UserConfig,
    private val activeWalletConnectRequest: ActiveWalletConnectRequest,
    private val securityService: GemSecurityService,
) {

    private val pauseTime = AtomicLong(0L)

    fun onPaused() {
        pauseTime.set(SystemClock.elapsedRealtime())
    }

    suspend fun shouldRelock(): Boolean = shouldRelock(now = SystemClock.elapsedRealtime())

    @VisibleForTesting
    internal suspend fun shouldRelock(now: Long): Boolean = securityService.shouldRelock(
        elapsedMilliseconds = now - pauseTime.get(),
        lockIntervalMinutes = userConfig.getLockInterval().first().toUInt(),
        authRequired = userConfig.authRequired(),
        hasPendingRequest = activeWalletConnectRequest.current.value != null,
    )

    @VisibleForTesting
    internal fun setPausedAt(time: Long) {
        pauseTime.set(time)
    }
}
