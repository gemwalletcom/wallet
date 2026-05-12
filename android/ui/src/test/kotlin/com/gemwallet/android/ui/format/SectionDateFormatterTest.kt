package com.gemwallet.android.ui.format

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Clock
import java.time.ZoneId
import java.time.ZonedDateTime
import java.util.Locale

class SectionDateFormatterTest {

    private val zone = ZoneId.of("UTC")
    private val clock = Clock.fixed(
        ZonedDateTime.of(2026, 5, 12, 10, 0, 0, 0, zone).toInstant(),
        zone,
    )
    private val locale = Locale.US

    @Test
    fun test_format() {
        assertEquals(TODAY, format(2026, 5, 12, 1))
        assertEquals(YESTERDAY, format(2026, 5, 11, 23))
        assertEquals("May 10, 2026", format(2026, 5, 10, 12))
        assertEquals("March 5, 2026", format(2026, 3, 5, 12))
    }

    private fun format(year: Int, month: Int, day: Int, hour: Int): String {
        val timestamp = ZonedDateTime.of(year, month, day, hour, 0, 0, 0, zone)
            .toInstant().toEpochMilli()
        return SectionDateFormatter.format(timestamp, TODAY, YESTERDAY, locale, clock)
    }

    companion object {
        private const val TODAY = "Today"
        private const val YESTERDAY = "Yesterday"
    }
}
