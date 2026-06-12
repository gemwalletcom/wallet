package com.gemwallet.android.ext

const val MILLIS_PER_SECOND = 1000L

fun Long.millisToSeconds(): Long = this / MILLIS_PER_SECOND

fun Long.secondsToMillis(): Long = this * MILLIS_PER_SECOND

fun currentTimestamp(): Long = System.currentTimeMillis().millisToSeconds()

fun nowSeconds(): ULong = currentTimestamp().toULong()
