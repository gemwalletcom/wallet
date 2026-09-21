package com.gemwallet.android.math

import uniffi.gemstone.GemNumberFormat
import java.math.BigDecimal
import java.text.DecimalFormatSymbols
import java.util.Locale

fun numberFormat(locale: Locale = Locale.getDefault()): GemNumberFormat = GemNumberFormat(DecimalFormatSymbols.getInstance(locale).decimalSeparator.toString())

fun String.plainInputNumber(): String = numberFormat().plain(this)

fun String.parseInputNumber(): BigDecimal = BigDecimal(plainInputNumber())

fun String.parseInputNumberOrNull(): BigDecimal? = try {
    parseInputNumber()
} catch (_: Throwable) {
    null
}
