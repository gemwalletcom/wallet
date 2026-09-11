package com.gemwallet.android.domains.duration

import androidx.test.ext.junit.runners.AndroidJUnit4
import junit.framework.TestCase.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import java.util.Locale

@RunWith(AndroidJUnit4::class)
class DurationFormatterTest {

    @Test
    fun estimatedConfirmation_readsMinutesWithTheLocaleUnit() {
        assertEquals("≈ 12 min", formatEstimatedConfirmation(12u, Locale.US))
        assertEquals("≈ 1 min", formatEstimatedConfirmation(1u, Locale.US))
        assertEquals("≈ 12 Min.", formatEstimatedConfirmation(12u, Locale.GERMANY))
    }
}
