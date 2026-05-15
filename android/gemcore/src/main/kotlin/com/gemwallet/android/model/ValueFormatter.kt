package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import com.wallet.core.primitives.Asset
import java.math.BigDecimal
import java.math.BigInteger
import java.math.MathContext
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale

class ValueFormatter(
    private val style: Style,
    private val locale: Locale = Locale.getDefault(),
    private val abbreviationThreshold: BigDecimal = ABBREVIATION_THRESHOLD,
) {
    enum class Style { Short, Compact, Auto, Full }

    fun string(value: BigInteger, asset: Asset): String =
        string(value, decimals = asset.decimals, currency = asset.symbol)

    fun string(value: BigInteger, decimals: Int, currency: String = ""): String =
        string(BigDecimal(value).movePointLeft(decimals), currency)

    fun string(value: BigDecimal, currency: String = ""): String {
        if (value.signum() == 0) return appendCurrency("0", currency)

        if (style == Style.Short && value.abs() >= abbreviationThreshold) {
            return appendCurrency(abbreviated(value), currency)
        }

        return appendCurrency(formatNumber(value), currency)
    }

    private fun formatNumber(decimal: BigDecimal): String {
        val formatter = (NumberFormat.getInstance(locale) as DecimalFormat).apply {
            roundingMode = RoundingMode.DOWN
        }
        return when (val precision = precision(decimal.abs())) {
            is Precision.Fraction -> formatter.run {
                minimumFractionDigits = precision.min
                maximumFractionDigits = precision.max
                format(decimal)
            }
            is Precision.Significant -> formatter.run {
                maximumFractionDigits = Int.MAX_VALUE
                format(decimal.round(MathContext(precision.max, RoundingMode.DOWN)).stripTrailingZeros())
            }
        }
    }

    private fun precision(magnitude: BigDecimal): Precision = when (style) {
        Style.Full -> Precision.full
        Style.Short -> if (magnitude >= SMALL_AMOUNT_THRESHOLD) Precision.twoPlaces else Precision.fourPlaces
        Style.Compact -> if (magnitude >= SMALL_AMOUNT_THRESHOLD) Precision.twoPlaces else Precision.fourPlaces
        Style.Auto -> when {
            magnitude >= BigDecimal.ONE -> Precision.twoPlaces
            magnitude >= DUST_THRESHOLD -> Precision.fourSignificant
            else -> Precision.full
        }
    }

    private fun abbreviated(decimal: BigDecimal): String {
        val formatter = CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT)
        formatter.maximumFractionDigits = 2
        return formatter.format(decimal)
    }

    private fun appendCurrency(value: String, currency: String): String =
        if (currency.isEmpty()) value else "$value $currency"

    private sealed interface Precision {
        data class Fraction(val min: Int, val max: Int) : Precision
        data class Significant(val max: Int) : Precision

        companion object {
            val full = Fraction(min = 0, max = 32)
            val twoPlaces = Fraction(min = 0, max = 2)
            val fourPlaces = Fraction(min = 0, max = 4)
            val fourSignificant = Significant(max = 4)
        }
    }

    companion object {
        private val SMALL_AMOUNT_THRESHOLD: BigDecimal = BigDecimal("0.1")
        private val DUST_THRESHOLD: BigDecimal = BigDecimal("0.0001")
        val ABBREVIATION_THRESHOLD: BigDecimal = BigDecimal(10_000)
    }
}
