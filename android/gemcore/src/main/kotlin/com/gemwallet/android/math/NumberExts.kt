package com.gemwallet.android.math

import java.math.BigInteger

fun BigInteger.multiplyByPercent(percent: Int): BigInteger =
    this * percent.toBigInteger() / BigInteger.valueOf(100)

fun ByteArray.toUnsignedInts(): List<Int> = map { it.toUByte().toInt() }
