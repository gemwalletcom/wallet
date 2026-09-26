package com.gemwallet.android.ui.format

import android.content.Context
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemChartDateStyle
import uniffi.gemstone.GemDay
import uniffi.gemstone.GemDayBoundaries
import uniffi.gemstone.GemDayLabel
import java.time.Clock
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

class SectionDateFormatter(private val todayLabel: String, private val yesterdayLabel: String, val boundaries: GemDayBoundaries = LocalDate.now().gemDay().boundaries()) {
    constructor(todayLabel: String, yesterdayLabel: String, clock: Clock) : this(
        todayLabel = todayLabel,
        yesterdayLabel = yesterdayLabel,
        boundaries = LocalDate.now(clock).gemDay().boundaries(),
    )

    fun format(date: LocalDate, locale: Locale): String = when (boundaries.label(date.gemDay())) {
        GemDayLabel.TODAY -> todayLabel

        GemDayLabel.YESTERDAY -> yesterdayLabel

        GemDayLabel.DATE ->
            DateTimeFormatter
                .ofLocalizedDate(FormatStyle.LONG)
                .withLocale(locale)
                .format(date)
    }

    fun section(timestamp: Long, zone: ZoneId, locale: Locale): String = if (timestamp == 0L) "" else format(Instant.ofEpochMilli(timestamp).atZone(zone).toLocalDate(), locale)

    fun row(timestamp: Long, zone: ZoneId, locale: Locale): String {
        if (timestamp == 0L) {
            return ""
        }
        val moment = Instant.ofEpochMilli(timestamp).atZone(zone)
        val time = DateTimeFormatter.ofLocalizedTime(FormatStyle.SHORT).withLocale(locale).format(moment)
        return when (boundaries.label(moment.toLocalDate().gemDay())) {
            GemDayLabel.TODAY -> "$todayLabel, $time"
            GemDayLabel.YESTERDAY -> "$yesterdayLabel, $time"
            GemDayLabel.DATE -> DateTimeFormatter.ofLocalizedDateTime(FormatStyle.LONG, FormatStyle.SHORT).withLocale(locale).format(moment)
        }
    }

    fun chartDate(timestamp: Long, style: GemChartDateStyle, zone: ZoneId, locale: Locale): String {
        if (timestamp == 0L) {
            return ""
        }
        val moment = Instant.ofEpochMilli(timestamp).atZone(zone)
        return when (style) {
            GemChartDateStyle.RELATIVE -> row(timestamp, zone, locale)
            GemChartDateStyle.DAY_TIME -> DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT).withLocale(locale).format(moment)
            GemChartDateStyle.DAY -> DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM).withLocale(locale).format(moment)
        }
    }
}

fun LocalDate.gemDay(): GemDay = GemDay(year = year, month = monthValue.toUInt(), day = dayOfMonth.toUInt())

fun GemDay.localDate(): LocalDate = LocalDate.of(year, month.toInt(), day.toInt())

fun Context.rowDateFormatter(): SectionDateFormatter = SectionDateFormatter(getString(R.string.date_today), getString(R.string.date_yesterday))
