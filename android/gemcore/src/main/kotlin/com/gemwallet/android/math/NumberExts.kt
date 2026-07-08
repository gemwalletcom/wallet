package com.gemwallet.android.math

import java.math.BigInteger

val MAX_256: BigInteger
    get() = BigInteger("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 16)

fun BigInteger.multiplyByPercent(percent: Int): BigInteger =
    this * percent.toBigInteger() / BigInteger.valueOf(100)