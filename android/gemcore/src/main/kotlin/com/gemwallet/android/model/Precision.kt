package com.gemwallet.android.model

import uniffi.gemstone.GemPrecision
import java.math.BigDecimal
import java.math.MathContext
import java.math.RoundingMode
import java.text.DecimalFormat
import uniffi.gemstone.adaptivePrecision as gemAdaptivePrecision

internal fun BigDecimal.rounded(precision: GemPrecision, roundingMode: RoundingMode): BigDecimal = when (precision) {
    is GemPrecision.Fraction -> setScale(precision.max.toInt(), roundingMode)
    is GemPrecision.Significant -> round(MathContext(precision.max.toInt(), roundingMode)).stripTrailingZeros()
}

internal fun DecimalFormat.format(value: BigDecimal, precision: GemPrecision): String = when (precision) {
    is GemPrecision.Fraction -> apply {
        minimumFractionDigits = precision.min.toInt()
        maximumFractionDigits = precision.max.toInt()
    }.format(value.rounded(precision, roundingMode))

    is GemPrecision.Significant -> apply {
        minimumFractionDigits = 0
        maximumFractionDigits = Int.MAX_VALUE
    }.format(value.rounded(precision, roundingMode))
}

internal fun adaptivePrecision(magnitude: BigDecimal): GemPrecision = gemAdaptivePrecision(magnitude.toDouble())
