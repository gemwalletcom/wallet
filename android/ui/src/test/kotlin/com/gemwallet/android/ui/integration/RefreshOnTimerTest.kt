package com.gemwallet.android.ui.integration

import android.os.Looper
import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.lifecycle.Lifecycle
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ui.components.RefreshOnTimer
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.Shadows.shadowOf
import java.time.Duration
import java.util.concurrent.atomic.AtomicInteger

@RunWith(AndroidJUnit4::class)
class RefreshOnTimerTest {
    @get:Rule
    val composeRule = createAndroidComposeRule<ComponentActivity>()

    @Test
    fun refreshesOnResumeWhenIntervalElapsedInBackground() {
        val refreshes = AtomicInteger()
        composeRule.mainClock.autoAdvance = false
        composeRule.setContent {
            RefreshOnTimer(2_000) { refreshes.incrementAndGet() }
        }
        passTime(2_000)
        assertEquals(1, refreshes.get())

        composeRule.activityRule.scenario.moveToState(Lifecycle.State.CREATED)
        passTime(2_500)
        assertEquals(1, refreshes.get())

        composeRule.activityRule.scenario.moveToState(Lifecycle.State.RESUMED)
        passTime(0)
        assertEquals(2, refreshes.get())
    }

    private fun passTime(millis: Long) {
        shadowOf(Looper.getMainLooper()).idleFor(Duration.ofMillis(millis))
        composeRule.mainClock.advanceTimeBy(millis)
    }
}
