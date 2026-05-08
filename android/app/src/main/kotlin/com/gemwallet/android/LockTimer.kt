package com.gemwallet.android

import android.os.SystemClock
import android.text.format.DateUtils
import androidx.annotation.VisibleForTesting
import com.gemwallet.android.data.repositories.config.UserConfig
import kotlinx.coroutines.flow.first
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

class LockTimer @Inject constructor(
    private val userConfig: UserConfig,
    private val activeRequestState: WalletConnectActiveRequestState,
) {

    private val pauseTime = AtomicLong(0L)

    fun onPaused() {
        pauseTime.set(SystemClock.elapsedRealtime())
    }

    suspend fun shouldRelock(): Boolean = shouldRelock(now = SystemClock.elapsedRealtime())

    @VisibleForTesting
    internal suspend fun shouldRelock(now: Long): Boolean {
        if (!userConfig.authRequired()) return false
        if (activeRequestState.hasActive.value) return false
        val elapsed = now - pauseTime.get()
        val lockIntervalMs = userConfig.getLockInterval().first() * DateUtils.MINUTE_IN_MILLIS
        return elapsed > lockIntervalMs
    }

    @VisibleForTesting
    internal fun setPausedAt(time: Long) {
        pauseTime.set(time)
    }
}
