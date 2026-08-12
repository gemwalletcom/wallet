package com.gemwallet.android.domains.duration

import org.junit.Assert.assertEquals
import org.junit.Test

class DurationFormatterTest {

    @Test
    fun estimatedConfirmation_formatsRoundedMinutes() {
        assertEquals("≈ 12 min", formatEstimatedConfirmation(720u))
        assertEquals("≈ 13 min", formatEstimatedConfirmation(750u))
    }

    @Test
    fun estimatedConfirmation_usesAtLeastOneMinute() {
        assertEquals("≈ 1 min", formatEstimatedConfirmation(1u))
    }
}
