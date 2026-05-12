package com.gemwallet.android.ui.format

import java.time.Clock
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale

class SectionDateFormatter(
    private val todayLabel: String,
    private val yesterdayLabel: String,
    private val clock: Clock = Clock.systemDefaultZone(),
) {

    fun format(date: LocalDate, locale: Locale): String {
        val today = LocalDate.now(clock)
        return when (date) {
            today -> todayLabel
            today.minusDays(1) -> yesterdayLabel
            else -> DateTimeFormatter
                .ofLocalizedDate(FormatStyle.LONG)
                .withLocale(locale)
                .format(date)
        }
    }
}
