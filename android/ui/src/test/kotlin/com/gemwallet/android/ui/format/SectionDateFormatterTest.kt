package com.gemwallet.android.ui.format

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemChartDateStyle
import java.time.Clock
import java.time.LocalDate
import java.time.ZoneId
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
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
        val today = at(14, 30, 12)
        val yesterday = at(9, 15, 11)
        val earlier = at(8, 5, 10)

        assertEquals("Today, ${time(today)}", formatter.row(millis(today), zone, locale))
        assertEquals("Yesterday, ${time(yesterday)}", formatter.row(millis(yesterday), zone, locale))
        assertEquals(dateTime(earlier, FormatStyle.LONG), formatter.row(millis(earlier), zone, locale))
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
        val earlier = at(8, 5, 10)

        assertEquals(dateTime(earlier, FormatStyle.LONG), formatter.chartDate(millis(earlier), GemChartDateStyle.RELATIVE, zone, locale))
        assertEquals(dateTime(earlier, FormatStyle.MEDIUM), formatter.chartDate(millis(earlier), GemChartDateStyle.DAY_TIME, zone, locale))
        assertEquals(date(earlier, FormatStyle.MEDIUM), formatter.chartDate(millis(earlier), GemChartDateStyle.DAY, zone, locale))
        assertEquals("", formatter.chartDate(0, GemChartDateStyle.DAY, zone, locale))
    }

    private fun at(hour: Int, minute: Int, day: Int): ZonedDateTime = ZonedDateTime.of(2026, 5, day, hour, minute, 0, 0, zone)

    private fun millis(moment: ZonedDateTime): Long = moment.toInstant().toEpochMilli()

    private fun time(moment: ZonedDateTime): String = DateTimeFormatter.ofLocalizedTime(FormatStyle.SHORT).withLocale(locale).format(moment)

    private fun dateTime(moment: ZonedDateTime, date: FormatStyle): String = DateTimeFormatter.ofLocalizedDateTime(date, FormatStyle.SHORT).withLocale(locale).format(moment)

    private fun date(moment: ZonedDateTime, style: FormatStyle): String = DateTimeFormatter.ofLocalizedDate(style).withLocale(locale).format(moment)

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
