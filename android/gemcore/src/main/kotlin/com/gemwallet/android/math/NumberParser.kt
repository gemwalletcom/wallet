package com.gemwallet.android.math

import java.math.BigDecimal

fun String.plainInputNumber(): String {
    val parts = trim().replace(",", ".")
        .replace(" ", "")
        .split(".")
    val number = List(parts.size) { i ->
        "${parts[i]}${if (i + 1 == parts.size - 1) "." else ""}"
    }.joinToString("")
    return number.trim().replace("\uFEFF", "")
}

fun String.parseInputNumber(): BigDecimal = BigDecimal(plainInputNumber())

fun String.parseInputNumberOrNull(): BigDecimal? {
    return try {
        parseInputNumber()
    } catch (_: Throwable) {
        null
    }
}
