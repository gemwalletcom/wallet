package com.gemwallet.android.ui.format

import org.junit.Assert.assertEquals
import org.junit.Test
import java.time.Clock
import java.time.LocalDate
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
    private val formatter = SectionDateFormatter(
        todayLabel = TODAY,
        yesterdayLabel = YESTERDAY,
        clock = clock,
    )

    @Test
    fun test_format() {
        assertEquals(TODAY, formatter.format(LocalDate.of(2026, 5, 12), locale))
        assertEquals(YESTERDAY, formatter.format(LocalDate.of(2026, 5, 11), locale))
        assertEquals("May 10, 2026", formatter.format(LocalDate.of(2026, 5, 10), locale))
        assertEquals("March 5, 2026", formatter.format(LocalDate.of(2026, 3, 5), locale))
    }

    companion object {
        private const val TODAY = "Today"
        private const val YESTERDAY = "Yesterday"
    }
}
