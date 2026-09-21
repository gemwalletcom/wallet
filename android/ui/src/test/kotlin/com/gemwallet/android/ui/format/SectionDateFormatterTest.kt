package com.gemwallet.android.ui.format

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemChartDateStyle
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
    fun `a row reads the day and the time, and nothing at all for no timestamp`() {
        val at = { hour: Int, minute: Int, day: Int -> ZonedDateTime.of(2026, 5, day, hour, minute, 0, 0, zone).toInstant().toEpochMilli() }

        assertEquals("Today, 2:30\u202fPM", formatter.row(at(14, 30, 12), zone, locale))
        assertEquals("Yesterday, 9:15\u202fAM", formatter.row(at(9, 15, 11), zone, locale))
        assertEquals("May 10, 2026, 8:05\u202fAM", formatter.row(at(8, 5, 10), zone, locale))
        assertEquals("", formatter.row(0, zone, locale))
    }

    @Test
    fun `a section reads the day alone`() {
        val at = { day: Int -> ZonedDateTime.of(2026, 5, day, 14, 30, 0, 0, zone).toInstant().toEpochMilli() }

        assertEquals(TODAY, formatter.section(at(12), zone, locale))
        assertEquals("May 10, 2026", formatter.section(at(10), zone, locale))
        assertEquals("", formatter.section(0, zone, locale))
    }

    @Test
    fun `a chart date follows the style Core picks for the period`() {
        val at = ZonedDateTime.of(2026, 5, 10, 8, 5, 0, 0, zone).toInstant().toEpochMilli()

        assertEquals("May 10, 2026, 8:05\u202fAM", formatter.chartDate(at, GemChartDateStyle.RELATIVE, zone, locale))
        assertEquals("May 10, 2026, 8:05\u202fAM", formatter.chartDate(at, GemChartDateStyle.DAY_TIME, zone, locale))
        assertEquals("May 10, 2026", formatter.chartDate(at, GemChartDateStyle.DAY, zone, locale))
        assertEquals("", formatter.chartDate(0, GemChartDateStyle.DAY, zone, locale))
    }

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
