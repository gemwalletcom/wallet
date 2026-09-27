package com.gemwallet.android.features.settings.viewmodels.lock

import android.os.SystemClock
import androidx.annotation.VisibleForTesting
import com.gemwallet.android.application.security.cases.SecurityPreferences
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemSecurityServiceInterface
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

class LockTimer @Inject constructor(private val securityPreferences: SecurityPreferences, private val securityService: GemSecurityServiceInterface) {

    private val pauseTime = AtomicLong(0L)

    fun onPaused() {
        pauseTime.set(SystemClock.elapsedRealtime())
    }

    suspend fun shouldRelock(): Boolean = shouldRelock(now = SystemClock.elapsedRealtime())

    @VisibleForTesting
    internal suspend fun shouldRelock(now: Long): Boolean = securityService.shouldRelock(
        elapsedMilliseconds = now - pauseTime.get(),
        lockIntervalMinutes = securityPreferences.getLockInterval().first().toUInt(),
        authRequired = securityPreferences.authRequired(),
    )
}
