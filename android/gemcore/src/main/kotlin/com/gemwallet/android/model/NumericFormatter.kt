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
    private val formatter = ThreadLocal.withInitial {
        (NumberFormat.getNumberInstance(locale) as DecimalFormat).apply { roundingMode = RoundingMode.HALF_EVEN }
    }

    fun string(
        value: Double,
        symbol: String? = null,
    ): String {
        if (!value.isFinite()) return ""
        val decimal = BigDecimal.valueOf(value)
        val number = formatter.get().format(decimal, adaptivePrecision(decimal.abs()))
        return if (symbol == null) number else "$number $symbol"
    }

    fun double(from: String): Double? {
        val text = from.trim()
        if (text.isEmpty()) return null
        return try {
            formatter.get().parse(text)?.toDouble()?.takeIf(Double::isFinite)
        } catch (_: ParseException) {
            null
        }
    }

}
