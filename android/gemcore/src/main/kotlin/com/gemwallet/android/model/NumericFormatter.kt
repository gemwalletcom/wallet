package com.gemwallet.android.model

import java.math.BigDecimal
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.text.ParseException
import java.util.Locale

class NumericFormatter(
    private val locale: Locale = Locale.getDefault(),
) {
    fun string(
        value: Double,
        symbol: String? = null,
    ): String {
        if (!value.isFinite()) return ""
        val decimal = BigDecimal.valueOf(value)
        val number = newFormatter().format(decimal, adaptivePrecision(decimal.abs()))
        return if (symbol == null) number else "$number $symbol"
    }

    fun double(from: String): Double? {
        val text = from.trim()
        if (text.isEmpty()) return null
        return try {
            newFormatter().parse(text)?.toDouble()?.takeIf(Double::isFinite)
        } catch (_: ParseException) {
            null
        }
    }

    private fun newFormatter(): DecimalFormat =
        (NumberFormat.getNumberInstance(locale) as DecimalFormat).apply {
            roundingMode = RoundingMode.HALF_EVEN
        }
}
