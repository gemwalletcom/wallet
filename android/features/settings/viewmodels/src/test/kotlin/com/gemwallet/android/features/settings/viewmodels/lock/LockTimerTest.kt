package com.gemwallet.android.features.settings.viewmodels.lock

import android.text.format.DateUtils
import com.gemwallet.android.application.security.cases.SecurityPreferences
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemSecurityService

class LockTimerTest {

    @Test
    fun shouldRelock_returnsFalseWhenAuthNotRequired() = runTest {
        val timer = lockTimer(authRequired = false, lockIntervalMinutes = 1)
        timer.onLeft(at = 0)

        assertFalse(timer.shouldRelockOnReturn(now = Long.MAX_VALUE))
    }

    @Test
    fun shouldRelock_returnsFalseWhenWithinLockInterval() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 1)
        timer.onLeft(at = 0)

        assertFalse(timer.shouldRelockOnReturn(now = DateUtils.MINUTE_IN_MILLIS))
    }

    @Test
    fun shouldRelock_returnsTrueAfterLockIntervalElapsed() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 1)
        timer.onLeft(at = 0)

        assertTrue(timer.shouldRelockOnReturn(now = DateUtils.MINUTE_IN_MILLIS + 1))
    }

    @Test
    fun shouldRelock_returnsTrueImmediatelyWhenIntervalIsZero() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 0)
        timer.onLeft(at = 0)

        assertTrue(timer.shouldRelockOnReturn(now = 1L))
    }

    @Test
    fun shouldRelock_returnsTrueWhileAWalletConnectRequestIsOpen() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 0)
        timer.onLeft(at = 0)

        assertTrue("an open request never holds the lock off", timer.shouldRelockOnReturn(now = Long.MAX_VALUE))
    }

    @Test
    fun returningWithoutLeavingNeverRelocks() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 0)

        assertFalse(timer.shouldRelockOnReturn(now = Long.MAX_VALUE))
    }

    @Test
    fun theFirstLeaveOfAnAwaySpellStartsTheLockPeriod() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 1)
        timer.onLeft(at = 0)
        timer.onLeft(at = DateUtils.MINUTE_IN_MILLIS)

        assertTrue(timer.shouldRelockOnReturn(now = DateUtils.MINUTE_IN_MILLIS + 1))
    }

    @Test
    fun returningEndsTheAwaySpell() = runTest {
        val timer = lockTimer(authRequired = true, lockIntervalMinutes = 1)
        timer.onLeft(at = 0)

        assertTrue(timer.shouldRelockOnReturn(now = DateUtils.MINUTE_IN_MILLIS + 1))
        assertFalse(timer.shouldRelockOnReturn(now = DateUtils.HOUR_IN_MILLIS))
    }

    private fun lockTimer(authRequired: Boolean, lockIntervalMinutes: Int): LockTimer {
        val securityPreferences = mockk<SecurityPreferences>()
        every { securityPreferences.isLockEnabled() } returns authRequired
        every { securityPreferences.getLockInterval() } returns flowOf(lockIntervalMinutes)
        return LockTimer(securityPreferences, GemSecurityService())
    }
}
