package com.gemwallet.android.features.settings.viewmodels.lock

import android.os.SystemClock
import androidx.annotation.VisibleForTesting
import com.gemwallet.android.application.security.cases.SecurityPreferences
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemSecurityServiceInterface
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

class LockTimer @Inject constructor(private val securityPreferences: SecurityPreferences, private val securityService: GemSecurityServiceInterface) {

    private val leftAt = AtomicLong(NOT_LEFT)

    fun onLeft(at: Long = SystemClock.elapsedRealtime()) {
        leftAt.compareAndSet(NOT_LEFT, at)
    }

    suspend fun shouldRelockOnReturn(): Boolean = shouldRelockOnReturn(now = SystemClock.elapsedRealtime())

    @VisibleForTesting
    internal suspend fun shouldRelockOnReturn(now: Long): Boolean {
        val left = leftAt.getAndSet(NOT_LEFT)
        if (left == NOT_LEFT) return false
        return securityService.shouldRelock(
            elapsedMilliseconds = now - left,
            lockIntervalMinutes = securityPreferences.getLockInterval().first().toUInt(),
            authRequired = securityPreferences.isLockEnabled(),
        )
    }

    private companion object {
        const val NOT_LEFT = -1L
    }
}
