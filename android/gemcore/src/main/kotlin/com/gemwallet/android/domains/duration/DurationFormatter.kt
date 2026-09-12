package com.gemwallet.android.domains.duration

import android.icu.text.MeasureFormat
import android.icu.util.Measure
import android.icu.util.MeasureUnit
import uniffi.gemstone.DurationFormatter
import uniffi.gemstone.GemDurationPart
import uniffi.gemstone.GemDurationUnit
import java.util.Locale

fun formatDuration(vararg measures: Measure, locale: Locale = Locale.getDefault()): String =
    MeasureFormat.getInstance(locale, MeasureFormat.FormatWidth.WIDE).formatMeasures(*measures)

fun formatAvailableIn(millis: Long, locale: Locale = Locale.getDefault()): String {
    val measures = DurationFormatter().countdownParts(millis / 1000).measures()
    return if (measures.isEmpty()) "" else formatDuration(*measures, locale = locale)
}

fun formatEstimatedConfirmation(seconds: UInt, locale: Locale = Locale.getDefault()): String {
    val measures = DurationFormatter().estimateParts(seconds.toLong()).measures()
    if (measures.isEmpty()) return ""
    val duration = MeasureFormat.getInstance(locale, MeasureFormat.FormatWidth.SHORT).formatMeasures(*measures)
    return "≈ $duration"
}

private fun List<GemDurationPart>.measures(): Array<Measure> =
    map { Measure(it.value, it.unit.measureUnit) }.toTypedArray()

private val GemDurationUnit.measureUnit: MeasureUnit
    get() = when (this) {
        GemDurationUnit.DAY -> MeasureUnit.DAY
        GemDurationUnit.HOUR -> MeasureUnit.HOUR
        GemDurationUnit.MINUTE -> MeasureUnit.MINUTE
        GemDurationUnit.SECOND -> MeasureUnit.SECOND
    }
