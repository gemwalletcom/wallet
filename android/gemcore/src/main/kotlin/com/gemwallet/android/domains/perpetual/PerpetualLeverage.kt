package com.gemwallet.android.domains.perpetual

import java.text.NumberFormat

private const val LeverageFractionDigits = 2

fun Int.formatLeverage(): String = "${this}x"

fun Double.formatLeverage(): String = NumberFormat.getNumberInstance()
    .apply {
        minimumFractionDigits = LeverageFractionDigits
        maximumFractionDigits = LeverageFractionDigits
    }
    .format(this) + "x"
