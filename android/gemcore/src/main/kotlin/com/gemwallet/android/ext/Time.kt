package com.gemwallet.android.ext

const val MILLIS_PER_SECOND = 1000L
const val SECONDS_PER_DAY = 86_400L

fun Long.millisToSeconds(): Long = this / MILLIS_PER_SECOND

fun Long.secondsToMillis(): Long = this * MILLIS_PER_SECOND

fun Long.secondsToDays(): Int = (this / SECONDS_PER_DAY).toInt()
