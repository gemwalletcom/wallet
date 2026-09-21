package com.gemwallet.android.ui.format

import uniffi.gemstone.GemDay
import uniffi.gemstone.GemDayBoundaries
import uniffi.gemstone.GemDayLabel
import java.time.Clock
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

class SectionDateFormatter(private val todayLabel: String, private val yesterdayLabel: String, private val boundaries: GemDayBoundaries = LocalDate.now().gemDay().boundaries()) {
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
}

fun LocalDate.gemDay(): GemDay = GemDay(year = year, month = monthValue.toUInt(), day = dayOfMonth.toUInt())
