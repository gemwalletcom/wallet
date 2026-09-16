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

internal fun adaptivePrecision(magnitude: BigDecimal): Precision = gemAdaptivePrecision(magnitude.toDouble()).toPrecision()

internal fun GemPrecision.toPrecision(): Precision = when (this) {
    is GemPrecision.Fraction -> Precision.Fraction(min = min.toInt(), max = max.toInt())
    is GemPrecision.Significant -> Precision.Significant(max = max.toInt())
}
