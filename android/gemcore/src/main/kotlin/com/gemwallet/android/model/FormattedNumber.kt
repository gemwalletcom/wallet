package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemNumberNotation
import uniffi.gemstone.GemNumberRounding
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemPrecision
import uniffi.gemstone.GemTransactionRowValue
import java.math.BigDecimal
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale

fun GemFormattedNumber.text(locale: Locale = Locale.getDefault()): String = when (notation) {
    GemNumberNotation.PARENTHESISED -> "(${body(locale)})"
    GemNumberNotation.PLAIN, GemNumberNotation.SIGNED -> body(locale)
}

private val GemFormattedNumber.showsSign: Boolean
    get() = notation == GemNumberNotation.SIGNED

private val GemFormattedNumber.numberRounding: RoundingMode
    get() = when (rounding) {
        GemNumberRounding.TO_NEAREST -> RoundingMode.HALF_EVEN
        GemNumberRounding.TOWARD_ZERO -> RoundingMode.DOWN
    }

private fun GemFormattedNumber.body(locale: Locale): String = when (val display = display) {
    is GemNumberDisplay.Number -> when (unit) {
        is GemNumberUnit.Percent -> percentText(BigDecimal.valueOf(value), display.precision, showsSign, numberRounding, locale)
        else -> appendSymbol(numberText(BigDecimal.valueOf(value), display.precision, locale))
    }

    is GemNumberDisplay.Abbreviated -> appendSymbol(abbreviatedText(BigDecimal.valueOf(value), locale))

    is GemNumberDisplay.BelowThreshold -> appendSymbol(
        "$signText<${numberText(BigDecimal.valueOf(display.threshold), GemPrecision.Fraction(display.places, display.places), locale, withSign = false)}",
    )
}

private val GemFormattedNumber.signText: String
    get() = when {
        !showsSign -> ""
        value < 0 -> "-"
        else -> "+"
    }

private val GemFormattedNumber.currencyCode: String?
    get() = (unit as? GemNumberUnit.Currency)?.code

private val GemFormattedNumber.symbol: String?
    get() = (unit as? GemNumberUnit.Symbol)?.symbol

private fun percentText(value: BigDecimal, precision: GemPrecision, showsSign: Boolean, rounding: RoundingMode, locale: Locale): String {
    val formatter = (NumberFormat.getPercentInstance(locale) as DecimalFormat).apply {
        when (precision) {
            is GemPrecision.Fraction -> {
                minimumFractionDigits = precision.min.toInt()
                maximumFractionDigits = precision.max.toInt()
            }

            is GemPrecision.Significant -> {
                minimumFractionDigits = 0
                maximumFractionDigits = Int.MAX_VALUE
            }
        }
        roundingMode = rounding
        val minus = decimalFormatSymbols.minusSign.toString()
        if (showsSign) {
            positivePrefix = "+" + positivePrefix
        } else {
            negativePrefix = negativePrefix.replace(minus, "")
        }
    }
    val amount = value.movePointLeft(2)
    return formatter.format(
        when (precision) {
            is GemPrecision.Fraction -> amount
            is GemPrecision.Significant -> amount.rounded(precision, rounding)
        },
    )
}

private fun GemFormattedNumber.appendSymbol(text: String): String = when (val unit = unit) {
    is GemNumberUnit.Symbol -> "$text ${unit.symbol}"
    GemNumberUnit.Multiplier -> "${text}x"
    is GemNumberUnit.Currency, GemNumberUnit.Percent, GemNumberUnit.Plain -> text
}

private fun GemFormattedNumber.numberText(value: BigDecimal, precision: GemPrecision, locale: Locale, withSign: Boolean = showsSign): String {
    val rounding = numberRounding
    val formatter = (numberFormat(locale) as DecimalFormat).apply {
        roundingMode = rounding
        if (withSign) {
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
        roundingMode = when (rounding) {
            GemNumberRounding.TO_NEAREST -> android.icu.math.BigDecimal.ROUND_HALF_EVEN
            GemNumberRounding.TOWARD_ZERO -> android.icu.math.BigDecimal.ROUND_DOWN
        }
        currencyCode?.let { currency = android.icu.util.Currency.getInstance(it) }
    }
    return when {
        showsSign -> "$signText${formatter.format(value.abs())}"
        else -> formatter.format(value)
    }
}

private fun GemFormattedNumber.numberFormat(locale: Locale): NumberFormat = currencyCode?.let { code ->
    NumberFormat.getCurrencyInstance(locale).apply { currency = java.util.Currency.getInstance(code) }
} ?: NumberFormat.getInstance(locale)

fun GemTransactionRowValue.text(): String? = when (this) {
    GemTransactionRowValue.None -> null
    is GemTransactionRowValue.AssetSymbol -> asset.symbol
    is GemTransactionRowValue.Number -> number.text()
}
