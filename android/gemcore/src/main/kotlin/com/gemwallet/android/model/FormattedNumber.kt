package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import java.math.BigDecimal
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberUnit

fun GemFormattedNumber.text(locale: Locale = Locale.getDefault()): String = when (notation) {
    GemNumberNotation.PARENTHESISED -> "(${body(locale)})"
    GemNumberNotation.PLAIN, GemNumberNotation.SIGNED -> body(locale)
}

private val GemFormattedNumber.showsSign: Boolean
    get() = notation == GemNumberNotation.SIGNED

private fun GemFormattedNumber.body(locale: Locale): String = when (val display = display) {
    is GemNumberDisplay.Number -> when (unit) {
        is GemNumberUnit.Percent -> percentText(BigDecimal.valueOf(value), display.precision.toPrecision(), showsSign, locale)
        else -> appendSymbol(numberText(BigDecimal.valueOf(value), display.precision.toPrecision(), locale))
    }
    is GemNumberDisplay.Abbreviated -> appendSymbol(abbreviatedText(BigDecimal.valueOf(value), locale))
    is GemNumberDisplay.BelowThreshold -> appendSymbol(
        "<${numberText(BigDecimal.valueOf(display.threshold), Precision.Fraction(display.places.toInt(), display.places.toInt()), locale)}"
    )
}

private val GemFormattedNumber.currencyCode: String?
    get() = (unit as? GemNumberUnit.Currency)?.code

private val GemFormattedNumber.symbol: String?
    get() = (unit as? GemNumberUnit.Symbol)?.symbol

private fun percentText(value: BigDecimal, precision: Precision, showsSign: Boolean, locale: Locale): String {
    val fraction = precision as Precision.Fraction
    val formatter = (NumberFormat.getPercentInstance(locale) as DecimalFormat).apply {
        minimumFractionDigits = fraction.min
        maximumFractionDigits = fraction.max
        roundingMode = RoundingMode.HALF_EVEN
        if (showsSign) {
            positivePrefix = "+"
        } else {
            positivePrefix = ""
            negativePrefix = ""
        }
    }
    return formatter.format(value.movePointLeft(2))
}

private fun GemFormattedNumber.appendSymbol(text: String): String =
    symbol?.let { "$text $it" } ?: text

private fun GemFormattedNumber.numberText(value: BigDecimal, precision: Precision, locale: Locale): String {
    val formatter = (numberFormat(locale) as DecimalFormat).apply {
        roundingMode = RoundingMode.HALF_EVEN
        if (showsSign) {
            positivePrefix = "+" + positivePrefix
        }
    }
    return formatter.format(value, precision)
}

private fun GemFormattedNumber.abbreviatedText(value: BigDecimal, locale: Locale): String {
    val formatter = CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT).apply {
        setSignificantDigitsUsed(false)
        minimumFractionDigits = 0
        maximumFractionDigits = 2
        roundingMode = android.icu.math.BigDecimal.ROUND_HALF_EVEN
        currencyCode?.let { currency = android.icu.util.Currency.getInstance(it) }
    }
    return formatter.format(value)
}

private fun GemFormattedNumber.numberFormat(locale: Locale): NumberFormat =
    currencyCode?.let { code ->
        NumberFormat.getCurrencyInstance(locale).apply { currency = java.util.Currency.getInstance(code) }
    } ?: NumberFormat.getInstance(locale)
