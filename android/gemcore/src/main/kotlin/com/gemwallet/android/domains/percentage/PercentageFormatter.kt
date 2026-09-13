package com.gemwallet.android.domains.percentage

import java.math.BigDecimal
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.GemPrecision

fun Double?.formatAsPercentage(
    style: GemPercentageStyle = GemPercentageStyle.SIGNED,
    locale: Locale = Locale.getDefault(),
): String = this.toPercentageDecimal()?.let { formatPercentage(it, style, locale) }.orEmpty()

private fun Double?.toPercentageDecimal(): BigDecimal? =
    this?.takeUnless { !it.isFinite() }?.let { BigDecimal.valueOf(it).movePointLeft(2) }

private fun formatPercentage(
    value: BigDecimal,
    style: GemPercentageStyle,
    locale: Locale,
): String {
    val format = style.format()
    val fraction = format.precision as GemPrecision.Fraction

    val formatter = (NumberFormat.getPercentInstance(locale) as DecimalFormat).apply {
        minimumFractionDigits = fraction.min.toInt()
        maximumFractionDigits = fraction.max.toInt()
        roundingMode = RoundingMode.HALF_EVEN

        if (format.showsSign) {
            positivePrefix = "+"
        } else {
            positivePrefix = ""
            negativePrefix = ""
        }
    }

    return formatter.format(if (format.showsSign) value else value.abs())
}
