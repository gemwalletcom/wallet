package com.gemwallet.android.math

class NumberSanitizer(
    private val maximumFractionDigits: Int? = null,
    private val maximumIntegerDigits: Int? = null,
    private val decimalSeparators: Set<Char> = setOf('.', ','),
) {
    fun sanitize(input: String): String {
        val filtered = input.filter { it.isDigit() || it in decimalSeparators }
        val separatorIndex = filtered.indexOfFirst { it in decimalSeparators }
        if (separatorIndex < 0) {
            return limitIntegerDigits(filtered)
        }
        val separator = filtered[separatorIndex]
        val integer = limitIntegerDigits(filtered.substring(0, separatorIndex))
        var fraction = filtered.substring(separatorIndex + 1).filter { it !in decimalSeparators }
        maximumFractionDigits?.let { fraction = fraction.take(it) }
        return "$integer$separator$fraction"
    }

    private fun limitIntegerDigits(value: String): String =
        maximumIntegerDigits?.let { value.take(it) } ?: value
}
