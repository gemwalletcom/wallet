package com.gemwallet.android.math

fun ByteArray.toUnsignedInts(): List<Int> = map { it.toUByte().toInt() }
