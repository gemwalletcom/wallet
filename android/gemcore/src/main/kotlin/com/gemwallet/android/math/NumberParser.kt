package com.gemwallet.android.math

import java.math.BigDecimal


fun String.parseInputNumber(): BigDecimal {
    val parts = trim().replace(",", ".")
        .replace(" ", "")
        .split(".")
    val number = List(parts.size) { i ->
        "${parts[i]}${if (i + 1 == parts.size - 1) "." else ""}"
    }.joinToString("")
    return BigDecimal(number.trim().replace("\uFEFF", ""))
}

fun String.parseInputNumberOrNull(): BigDecimal? {
    return try {
        parseInputNumber()
    } catch (_: Throwable) {
        null
    }
}