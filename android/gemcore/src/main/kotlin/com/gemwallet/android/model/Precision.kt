package com.gemwallet.android.model

import java.math.BigDecimal
import java.math.MathContext
import java.math.RoundingMode
import java.text.DecimalFormat
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.adaptivePrecision as gemAdaptivePrecision

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

internal fun BigDecimal.rounded(precision: Precision, roundingMode: RoundingMode): BigDecimal = when (precision) {
    is Precision.Fraction -> setScale(precision.max, roundingMode)
    is Precision.Significant -> round(MathContext(precision.max, roundingMode)).stripTrailingZeros()
}

internal fun DecimalFormat.format(value: BigDecimal, precision: Precision): String = when (precision) {
    is Precision.Fraction -> apply {
        minimumFractionDigits = precision.min
        maximumFractionDigits = precision.max
    }.format(value.rounded(precision, roundingMode))
    is Precision.Significant -> apply {
        minimumFractionDigits = 0
        maximumFractionDigits = Int.MAX_VALUE
    }.format(value.rounded(precision, roundingMode))
}

internal fun adaptivePrecision(magnitude: BigDecimal): Precision = when (val precision = gemAdaptivePrecision(magnitude.toDouble())) {
    is GemPrecision.Fraction -> Precision.Fraction(min = precision.min.toInt(), max = precision.max.toInt())
    is GemPrecision.Significant -> Precision.Significant(max = precision.max.toInt())
}

internal val ABBREVIATION_THRESHOLD: BigDecimal = BigDecimal(100_000)
