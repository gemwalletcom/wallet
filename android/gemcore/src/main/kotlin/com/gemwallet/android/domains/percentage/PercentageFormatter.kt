package com.gemwallet.android.domains.percentage

import java.math.BigDecimal
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale

enum class PercentageFormatterStyle {
    Percent,
    PercentSignLess,
    PercentSignLessCompact,
}

fun Double?.formatAsPercentage(
    style: PercentageFormatterStyle = PercentageFormatterStyle.Percent,
    locale: Locale = Locale.getDefault(),
): String = this.toPercentageDecimal()?.let { formatPercentage(it, style, locale) }.orEmpty()

private fun Double?.toPercentageDecimal(): BigDecimal? =
    this?.takeUnless { !it.isFinite() }?.let { BigDecimal.valueOf(it).movePointLeft(2) }

private fun formatPercentage(
    value: BigDecimal,
    style: PercentageFormatterStyle,
    locale: Locale,
): String {
    val minimumFractionDigits = if (style == PercentageFormatterStyle.PercentSignLessCompact) 0 else 2
    val isSignVisible = style == PercentageFormatterStyle.Percent

    val formatter = (NumberFormat.getPercentInstance(locale) as DecimalFormat).apply {
        setMinimumFractionDigits(minimumFractionDigits)
        setMaximumFractionDigits(2)
        setRoundingMode(RoundingMode.HALF_EVEN)

        if (isSignVisible) {
            positivePrefix = "+"
        } else {
            positivePrefix = ""
            negativePrefix = ""
        }
    }

    val displayValue = if (isSignVisible) value else value.abs()
    return formatter.format(displayValue)
}
