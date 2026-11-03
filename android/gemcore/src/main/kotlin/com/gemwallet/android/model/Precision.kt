package com.gemwallet.android.model

import android.icu.text.DecimalFormat
import uniffi.gemstone.GemPrecision
import java.math.BigDecimal
import java.math.MathContext
import java.math.RoundingMode

internal fun BigDecimal.rounded(precision: GemPrecision, roundingMode: RoundingMode): BigDecimal = when (precision) {
    is GemPrecision.Fraction -> setScale(precision.max, roundingMode)
    is GemPrecision.Significant -> round(MathContext(precision.max, roundingMode)).stripTrailingZeros()
}

internal fun DecimalFormat.format(value: BigDecimal, precision: GemPrecision): String = when (precision) {
    is GemPrecision.Fraction -> apply {
        minimumFractionDigits = precision.min
        maximumFractionDigits = precision.max
    }.format(value.rounded(precision, RoundingMode.valueOf(roundingMode)))

    is GemPrecision.Significant -> apply {
        minimumFractionDigits = 0
        maximumFractionDigits = Int.MAX_VALUE
    }.format(value.rounded(precision, RoundingMode.valueOf(roundingMode)))
}
