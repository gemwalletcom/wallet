package com.gemwallet.android.domains.duration

import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.concurrent.TimeUnit

class DurationPartsTest {

    @Test
    fun over24Hours_showsDaysAndHours() {
        assertEquals(
            listOf(5L to DurationUnit.DAY, 3L to DurationUnit.HOUR),
            availableInParts(TimeUnit.DAYS.toMillis(5) + TimeUnit.HOURS.toMillis(3)),
        )
    }

    @Test
    fun under24Hours_showsHoursAndMinutes() {
        assertEquals(
            listOf(23L to DurationUnit.HOUR, 59L to DurationUnit.MINUTE),
            availableInParts(TimeUnit.HOURS.toMillis(23) + TimeUnit.MINUTES.toMillis(59)),
        )
    }

    @Test
    fun under1Hour_dropsLeadingZeroHour() {
        assertEquals(
            listOf(45L to DurationUnit.MINUTE),
            availableInParts(TimeUnit.MINUTES.toMillis(45)),
        )
    }

    @Test
    fun subMinute_keepsMinutesNeverSeconds() {
        assertEquals(
            listOf(0L to DurationUnit.MINUTE),
            availableInParts(TimeUnit.SECONDS.toMillis(30)),
        )
    }

    @Test
    fun negative_isEmpty() {
        assertEquals(emptyList<Pair<Long, DurationUnit>>(), availableInParts(-1L))
    }
}
