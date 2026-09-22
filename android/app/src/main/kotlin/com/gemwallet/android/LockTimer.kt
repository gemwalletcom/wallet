package com.gemwallet.android

import android.os.SystemClock
import androidx.annotation.VisibleForTesting
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemSecurityService
import uniffi.gemstone.GemSecurityServiceInterface
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

class LockTimer @Inject constructor(private val userConfig: UserConfig, private val securityService: GemSecurityServiceInterface) {

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
    )
}
