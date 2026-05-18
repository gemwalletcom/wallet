package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import com.wallet.core.primitives.Asset
import java.math.BigDecimal
import java.math.BigInteger
import java.math.MathContext
import java.util.Locale

class Crypto(val atomicValue: BigInteger) {
    constructor(value: String, decimals: Int) : this(value.toBigDecimal(), decimals)

    constructor(value: String) : this(value.toBigInteger())

    constructor(value: BigDecimal, decimals: Int) : this(
        value.multiply(BigDecimal.TEN.pow(decimals)).toBigInteger()
    )

    fun convert(decimals: Int, price: Double): Fiat {
        val result = atomicValue.toBigDecimal()
            .divide(BigDecimal.TEN.pow(decimals), MathContext.DECIMAL128)
            .multiply(price.toBigDecimal())
        return Fiat(result)
    }

    fun value(decimals: Int): BigDecimal =
        atomicValue.toBigDecimal().divide(BigDecimal.TEN.pow(decimals), MathContext.DECIMAL128)
}

class Fiat(val atomicValue: BigDecimal) {
    fun convert(decimals: Int, price: Double): Crypto {
        if (price == 0.0) {
            return Crypto(BigInteger.ZERO)
        }
        val result = atomicValue.divide(price.toBigDecimal(), MathContext.DECIMAL128)
            .multiply(BigDecimal.TEN.pow(decimals))
            .toBigInteger()
        return Crypto(result)
    }

    @Suppress("UNUSED_PARAMETER")
    fun value(decimals: Int): BigDecimal = atomicValue
}

fun Asset.compactFormatter(
    value: Double,
    locale: Locale = Locale.getDefault(),
): String {
    val formatter = CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT)
    formatter.maximumFractionDigits = 2
    return "${formatter.format(value)} $symbol"
}

fun Asset.formatSupply(
    value: Double,
    locale: Locale = Locale.getDefault(),
): String = if (value == 0.0) "∞ $symbol" else compactFormatter(value = value, locale = locale)
