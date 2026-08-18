package com.gemwallet.android.model

import java.math.BigDecimal
import java.math.BigInteger
import java.math.MathContext

class Crypto(val atomicValue: BigInteger) {
    constructor(value: String, decimals: Int) : this(value.toBigDecimal(), decimals)

    constructor(value: String) : this(value.toBigInteger())

    constructor(value: BigDecimal, decimals: Int) : this(
        value.multiply(BigDecimal.TEN.pow(decimals)).toBigInteger()
    )

    fun value(decimals: Int): BigDecimal =
        atomicValue.toBigDecimal().divide(BigDecimal.TEN.pow(decimals), MathContext.DECIMAL128)

    companion object {
        fun exact(value: String, decimals: Int): Crypto? = runCatching {
            Crypto(value.toBigDecimal().multiply(BigDecimal.TEN.pow(decimals)).toBigIntegerExact())
        }.getOrNull()
    }
}

class Fiat(val atomicValue: BigDecimal)
