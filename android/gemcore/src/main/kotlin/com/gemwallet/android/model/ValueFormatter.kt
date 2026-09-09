package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import com.wallet.core.primitives.Asset
import java.math.BigDecimal
import java.math.BigInteger
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale

class ValueFormatter(
    private val style: Style,
    private val locale: Locale = Locale.getDefault(),
    private val abbreviationThreshold: BigDecimal = ABBREVIATION_THRESHOLD,
) {
    enum class Style { Full, Short, Auto }

    private val decimalFormatter = ThreadLocal.withInitial {
        (NumberFormat.getInstance(locale) as DecimalFormat).apply { roundingMode = ROUNDING_MODE }
    }

    private val compactFormatter = ThreadLocal.withInitial {
        CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT).apply {
            setSignificantDigitsUsed(false)
            minimumFractionDigits = 0
            maximumFractionDigits = 2
            roundingMode = android.icu.math.BigDecimal.ROUND_DOWN
        }
    }

    fun string(value: BigInteger, asset: Asset): String =
        string(value, decimals = asset.decimals, currency = asset.symbol)

    fun string(value: BigInteger, decimals: Int, currency: String = ""): String =
        string(BigDecimal(value).movePointLeft(decimals), currency)

    fun string(value: BigDecimal, currency: String = ""): String {
        if (value.signum() == 0) return appendCurrency("0", currency)

        if (style == Style.Short && value.abs() >= abbreviationThreshold) {
            return appendCurrency(abbreviated(value), currency)
        }

        if (style == Style.Short && value.abs() < DUST_THRESHOLD) {
            return appendCurrency("<${formattedDustThreshold()}", currency)
        }

        return appendCurrency(decimalFormatter.get().format(value, precision(value.abs())), currency)
    }

    fun rounded(value: BigDecimal): BigDecimal = value.rounded(precision(value.abs()), ROUNDING_MODE)

    private fun precision(magnitude: BigDecimal): Precision = when (style) {
        Style.Full -> Precision.full
        Style.Short -> if (magnitude >= SMALL_AMOUNT_THRESHOLD) Precision.upToTwoPlaces else Precision.upToFourPlaces
        Style.Auto -> if (magnitude >= BigDecimal.ONE) Precision.upToTwoPlaces else Precision.fourSignificant
    }

    private fun abbreviated(decimal: BigDecimal): String = compactFormatter.get().format(decimal)

    private fun formattedDustThreshold(): String = decimalFormatter.get().format(DUST_THRESHOLD, Precision.Fraction(min = 4, max = 4))

    private fun appendCurrency(value: String, currency: String): String =
        if (currency.isEmpty()) value else "$value $currency"

    companion object {
        private val ROUNDING_MODE: RoundingMode = RoundingMode.DOWN
        private val SMALL_AMOUNT_THRESHOLD: BigDecimal = BigDecimal("0.1")
        private val DUST_THRESHOLD: BigDecimal = BigDecimal("0.0001")
        val ABBREVIATION_THRESHOLD: BigDecimal = BigDecimal(100_000)
    }
}
