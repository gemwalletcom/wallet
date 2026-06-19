package com.gemwallet.android.model

import java.math.BigDecimal
import java.math.MathContext
import java.text.DecimalFormat

internal sealed interface Precision {
    data class Fraction(val min: Int, val max: Int) : Precision
    data class Significant(val max: Int) : Precision

    companion object {
        val twoPlaces = Fraction(min = 2, max = 2)
        val upToTwoPlaces = Fraction(min = 0, max = 2)
        val upToFourPlaces = Fraction(min = 0, max = 4)
        val fourSignificant = Significant(max = 4)
        val full = Fraction(min = 0, max = 32)
    }
}

internal fun DecimalFormat.format(value: BigDecimal, precision: Precision): String = when (precision) {
    is Precision.Fraction -> apply {
        minimumFractionDigits = precision.min
        maximumFractionDigits = precision.max
    }.format(value)
    is Precision.Significant -> apply {
        minimumFractionDigits = 0
        maximumFractionDigits = Int.MAX_VALUE
    }.format(value.round(MathContext(precision.max, roundingMode)).stripTrailingZeros())
}

internal fun adaptivePrecision(magnitude: BigDecimal): Precision =
    if (magnitude < DUST_THRESHOLD || magnitude >= SMALL_VALUE_THRESHOLD) {
        Precision.twoPlaces
    } else {
        Precision.fourSignificant
    }

internal val ABBREVIATION_THRESHOLD: BigDecimal = BigDecimal(100_000)

private val SMALL_VALUE_THRESHOLD: BigDecimal = BigDecimal("0.99")
private val DUST_THRESHOLD: BigDecimal = BigDecimal("0.0000000001")
