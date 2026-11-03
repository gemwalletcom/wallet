package com.gemwallet.android.model

import android.icu.math.BigDecimal.ROUND_DOWN
import android.icu.math.BigDecimal.ROUND_HALF_EVEN
import android.icu.math.BigDecimal.ROUND_UP
import android.icu.text.CompactDecimalFormat
import android.icu.text.DecimalFormat
import android.icu.text.NumberFormat
import android.icu.util.Currency
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberRounding
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemTransactionRowValue
import java.math.BigDecimal
import java.math.RoundingMode
import java.util.Locale
import java.util.concurrent.ConcurrentHashMap

fun GemFormattedNumber.text(locale: Locale = Locale.getDefault()): String = when (notation) {
    GemNumberNotation.PARENTHESISED -> "(${body(locale)})"
    GemNumberNotation.PLAIN, GemNumberNotation.SIGNED -> body(locale)
}

private val GemFormattedNumber.showsSign: Boolean
    get() = notation == GemNumberNotation.SIGNED

private val GemFormattedNumber.numberRounding: Int
    get() = when (rounding) {
        GemNumberRounding.TO_NEAREST -> ROUND_HALF_EVEN
        GemNumberRounding.TOWARD_ZERO -> ROUND_DOWN
        GemNumberRounding.AWAY_FROM_ZERO -> ROUND_UP
    }

private fun GemFormattedNumber.body(locale: Locale): String = when (val display = display) {
    is GemNumberDisplay.Number -> when (unit) {
        is GemNumberUnit.Percent -> percentText(BigDecimal.valueOf(value), display.precision, showsSign, numberRounding, locale)
        is GemNumberUnit.Currency, is GemNumberUnit.Symbol, GemNumberUnit.Plain, GemNumberUnit.Multiplier -> appendSymbol(numberText(decimalValue, display.precision, locale))
    }

    is GemNumberDisplay.Abbreviated -> appendSymbol(abbreviatedText(BigDecimal.valueOf(value), locale))

    is GemNumberDisplay.BelowThreshold -> appendSymbol(
        "$signText<${numberText(BigDecimal.valueOf(display.threshold), GemPrecision.Fraction(display.places, display.places), locale, withSign = false)}",
    )
}

private val GemFormattedNumber.decimalValue: BigDecimal
    get() = exact?.let { BigDecimal(it).let { magnitude -> if (value < 0) magnitude.negate() else magnitude } } ?: BigDecimal.valueOf(value)

private val GemFormattedNumber.signText: String
    get() = when {
        !showsSign -> ""
        value < 0 -> "-"
        else -> "+"
    }

private val GemFormattedNumber.currencyCode: String?
    get() = (unit as? GemNumberUnit.Currency)?.code

private fun percentText(value: BigDecimal, precision: GemPrecision, showsSign: Boolean, rounding: Int, locale: Locale): String {
    val formatter = (percentFormats.getOrPut(locale to showsSign) { percentFormat(locale, showsSign) }.clone() as DecimalFormat).apply {
        when (precision) {
            is GemPrecision.Fraction -> {
                minimumFractionDigits = precision.min
                maximumFractionDigits = precision.max
            }

            is GemPrecision.Significant -> {
                minimumFractionDigits = 0
                maximumFractionDigits = Int.MAX_VALUE
            }
        }
        roundingMode = rounding
    }
    val amount = value.movePointLeft(2)
    return formatter.format(
        when (precision) {
            is GemPrecision.Fraction -> amount
            is GemPrecision.Significant -> amount.rounded(precision, RoundingMode.valueOf(rounding))
        },
    )
}

private fun GemFormattedNumber.appendSymbol(text: String): String = when (val unit = unit) {
    is GemNumberUnit.Symbol -> "$text ${unit.symbol}"
    GemNumberUnit.Multiplier -> "${text}x"
    is GemNumberUnit.Currency, GemNumberUnit.Percent, GemNumberUnit.Plain -> text
}

private fun GemFormattedNumber.numberText(value: BigDecimal, precision: GemPrecision, locale: Locale, withSign: Boolean = showsSign): String {
    val formatter = numberFormat(locale, withSign).apply {
        roundingMode = numberRounding
    }
    return formatter.format(value, precision)
}

private fun GemFormattedNumber.abbreviatedText(value: BigDecimal, locale: Locale): String {
    val formatter = CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT).apply {
        setSignificantDigitsUsed(false)
        minimumFractionDigits = 0
        maximumFractionDigits = 2
        roundingMode = numberRounding
        currencyCode?.let { currency = Currency.getInstance(it) }
    }
    return when {
        showsSign -> "$signText${formatter.format(value.abs())}"
        else -> formatter.format(value)
    }
}

private val numberFormats = ConcurrentHashMap<Triple<Locale, String?, Boolean>, DecimalFormat>()

private val percentFormats = ConcurrentHashMap<Pair<Locale, Boolean>, DecimalFormat>()

private fun GemFormattedNumber.numberFormat(locale: Locale, withSign: Boolean): DecimalFormat = numberFormats.getOrPut(Triple(locale, currencyCode, withSign)) {
    ((currencyCode?.let { code -> NumberFormat.getCurrencyInstance(locale).apply { currency = Currency.getInstance(code) } } ?: NumberFormat.getInstance(locale)) as DecimalFormat).apply {
        if (withSign) {
            alwaysShowSign()
        }
    }
}.clone() as DecimalFormat

private fun percentFormat(locale: Locale, showsSign: Boolean): DecimalFormat = (NumberFormat.getPercentInstance(locale) as DecimalFormat).apply {
    if (showsSign) {
        alwaysShowSign()
    } else {
        negativePrefix = negativePrefix.replace(decimalFormatSymbols.minusSignString, "")
    }
}

private fun DecimalFormat.alwaysShowSign() {
    val pattern = toPattern()
    val negative = pattern.substringAfter(';', "-$pattern")
    applyPattern("${negative.replace('-', '+')};$negative")
}

fun GemTransactionRowValue.text(): String? = when (this) {
    GemTransactionRowValue.None -> null
    is GemTransactionRowValue.AssetSymbol -> asset.symbol
    is GemTransactionRowValue.Number -> number.text()
}
