package com.gemwallet.android.domains.perpetual

import java.text.NumberFormat
import uniffi.gemstone.GemPerpetual
import uniffi.gemstone.PerpetualProvider

private const val LeverageFractionDigits = 2

fun Int.formatLeverage(): String = GemPerpetual(PerpetualProvider.HYPERCORE).use { it.leverageText(toUByte()) }

fun Double.formatLeverage(): String = NumberFormat.getNumberInstance()
    .apply {
        minimumFractionDigits = LeverageFractionDigits
        maximumFractionDigits = LeverageFractionDigits
    }
    .format(this) + "x"
