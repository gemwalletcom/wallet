package com.gemwallet.android.domains.duration

private const val MINUTE_MS = 60_000L
private const val HOUR_MS = 60 * MINUTE_MS
private const val DAY_MS = 24 * HOUR_MS

internal enum class DurationUnit { DAY, HOUR, MINUTE }

internal fun availableInParts(millis: Long): List<Pair<Long, DurationUnit>> {
    if (millis < 0) return emptyList()
    val parts = if (millis < DAY_MS) {
        listOf(
            millis / HOUR_MS to DurationUnit.HOUR,
            (millis % HOUR_MS) / MINUTE_MS to DurationUnit.MINUTE,
        )
    } else {
        listOf(
            millis / DAY_MS to DurationUnit.DAY,
            (millis % DAY_MS) / HOUR_MS to DurationUnit.HOUR,
        )
    }
    return parts.dropWhile { it.first == 0L }.ifEmpty { parts.takeLast(1) }
}
