package com.gemwallet.android.ui.components

import android.os.SystemClock
import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.lifecycle.Lifecycle
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import java.util.concurrent.atomic.AtomicInteger

class RefreshOnTimerTest {
    @get:Rule
    val composeRule = createAndroidComposeRule<ComponentActivity>()

    @Test
    fun refreshesOnResumeWhenIntervalElapsedInBackground() {
        val refreshes = AtomicInteger()
        composeRule.setContent {
            RefreshOnTimer(2_000) { refreshes.incrementAndGet() }
        }
        composeRule.waitUntil(timeoutMillis = 5_000) { refreshes.get() == 1 }
        composeRule.activityRule.scenario.moveToState(Lifecycle.State.CREATED)
        val stoppedAt = SystemClock.elapsedRealtime()
        composeRule.waitUntil(timeoutMillis = 5_000) {
            SystemClock.elapsedRealtime() - stoppedAt >= 2_500
        }
        assertEquals(1, refreshes.get())

        composeRule.activityRule.scenario.moveToState(Lifecycle.State.RESUMED)
        composeRule.waitUntil(timeoutMillis = 1_000) { refreshes.get() == 2 }
    }
}
