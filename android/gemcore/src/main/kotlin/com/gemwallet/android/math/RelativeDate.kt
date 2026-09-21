package com.gemwallet.android.math

import android.text.format.DateUtils
import uniffi.gemstone.GemChartDateStyle
import java.text.DateFormat
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date

fun getRelativeDate(timestamp: Long): String {
    if (timestamp == 0L) {
        return ""
    }
    return if (DateUtils.isToday(timestamp) || DateUtils.isToday(timestamp + DateUtils.DAY_IN_MILLIS)) {
        DateUtils.getRelativeTimeSpanString(
            timestamp,
            System.currentTimeMillis(),
            DateUtils.DAY_IN_MILLIS,
        ).toString() +
            " " + DateFormat.getTimeInstance(DateFormat.SHORT)
                .format(Date(timestamp))
    } else {
        val createdAt = Calendar.getInstance()
        createdAt.timeInMillis = timestamp
        (DateFormat.getDateTimeInstance(DateFormat.LONG, DateFormat.SHORT) as SimpleDateFormat)
            .format(createdAt.time)
    }
}

fun getChartDate(timestamp: Long, style: GemChartDateStyle): String {
    if (timestamp == 0L) {
        return ""
    }
    return when (style) {
        GemChartDateStyle.RELATIVE -> getRelativeDate(timestamp)
        GemChartDateStyle.DAY_TIME -> DateFormat.getDateTimeInstance(DateFormat.MEDIUM, DateFormat.SHORT).format(Date(timestamp))
        GemChartDateStyle.DAY -> DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(timestamp))
    }
}
