package com.gemwallet.android.ext

fun currentTimestamp(): Long = System.currentTimeMillis() / 1000

fun nowSeconds(): ULong = currentTimestamp().toULong()
