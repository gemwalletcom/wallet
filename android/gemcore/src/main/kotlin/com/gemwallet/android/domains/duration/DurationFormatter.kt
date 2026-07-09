package com.gemwallet.android.domains.duration

import android.icu.text.MeasureFormat
import android.icu.util.Measure
import android.icu.util.MeasureUnit
import java.util.Locale

fun formatDuration(vararg measures: Measure, locale: Locale = Locale.getDefault()): String =
    MeasureFormat.getInstance(locale, MeasureFormat.FormatWidth.WIDE).formatMeasures(*measures)

fun formatAvailableIn(millis: Long, locale: Locale = Locale.getDefault()): String {
    val measures = availableInParts(millis).map { Measure(it.first, it.second.measureUnit) }
    return if (measures.isEmpty()) "" else formatDuration(*measures.toTypedArray(), locale = locale)
}

private val DurationUnit.measureUnit: MeasureUnit
    get() = when (this) {
        DurationUnit.DAY -> MeasureUnit.DAY
        DurationUnit.HOUR -> MeasureUnit.HOUR
        DurationUnit.MINUTE -> MeasureUnit.MINUTE
    }
