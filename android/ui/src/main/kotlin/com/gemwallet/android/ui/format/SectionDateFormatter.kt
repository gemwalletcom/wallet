package com.gemwallet.android.ui.format

import java.time.Clock
import java.time.Instant
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

object SectionDateFormatter {

    fun format(
        timestamp: Long,
        todayLabel: String,
        yesterdayLabel: String,
        locale: Locale,
        clock: Clock = Clock.systemDefaultZone(),
    ): String = format(
        date = Instant.ofEpochMilli(timestamp).atZone(clock.zone).toLocalDate(),
        todayLabel = todayLabel,
        yesterdayLabel = yesterdayLabel,
        locale = locale,
        today = LocalDate.now(clock),
    )

    fun format(
        date: LocalDate,
        todayLabel: String,
        yesterdayLabel: String,
        locale: Locale,
        today: LocalDate = LocalDate.now(),
    ): String = when (date) {
        today -> todayLabel
        today.minusDays(1) -> yesterdayLabel
        else -> DateTimeFormatter
            .ofLocalizedDate(FormatStyle.LONG)
            .withLocale(locale)
            .format(date)
    }
}
