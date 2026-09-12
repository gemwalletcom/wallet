package com.gemwallet.android.math

import uniffi.gemstone.GemNumberFormat
import java.math.BigDecimal
import java.text.DecimalFormatSymbols

fun String.plainInputNumber(): String = GemNumberFormat(DecimalFormatSymbols.getInstance().decimalSeparator.toString()).plain(this)

fun String.parseInputNumber(): BigDecimal = BigDecimal(plainInputNumber())

fun String.parseInputNumberOrNull(): BigDecimal? {
    return try {
        parseInputNumber()
    } catch (_: Throwable) {
        null
    }
}
