package com.gemwallet.android.math

import java.math.BigInteger

fun BigInteger.multiplyByPercent(percent: Int): BigInteger =
    this * percent.toBigInteger() / BigInteger.valueOf(100)
