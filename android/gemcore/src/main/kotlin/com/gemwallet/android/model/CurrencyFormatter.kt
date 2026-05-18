package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import com.wallet.core.primitives.Currency
import java.math.BigDecimal
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale

class CurrencyFormatter(
    private val type: Type = Type.Currency,
    private val currency: Currency,
    private val locale: Locale = Locale.getDefault(),
) {
    enum class Type { Currency, Fiat, Abbreviated }

    private val currencyFormatter: DecimalFormat by lazy {
        (NumberFormat.getCurrencyInstance(locale) as DecimalFormat).apply {
            currency = java.util.Currency.getInstance(this@CurrencyFormatter.currency.string)
            roundingMode = RoundingMode.HALF_EVEN
        }
    }

    private val abbreviatedFormatter: CompactDecimalFormat by lazy {
        CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT).apply {
            currency = android.icu.util.Currency.getInstance(this@CurrencyFormatter.currency.string)
            maximumFractionDigits = 2
        }
    }

    fun string(value: Double): String = string(BigDecimal.valueOf(value))

    fun string(value: BigDecimal): String =
        if (type == Type.Abbreviated && value.abs() >= ABBREVIATION_THRESHOLD) {
            abbreviatedFormatter.format(value)
        } else {
            currencyFormatter.format(value, precision(value.abs()))
        }

    private fun precision(magnitude: BigDecimal): Precision = when {
        type == Type.Fiat -> Precision.twoPlaces
        magnitude < DUST_THRESHOLD || magnitude >= SMALL_THRESHOLD -> Precision.twoPlaces
        else -> Precision.fourSignificant
    }

    private companion object {
        val SMALL_THRESHOLD: BigDecimal = BigDecimal("0.99")
        val DUST_THRESHOLD: BigDecimal = BigDecimal("0.0000000001")
        val ABBREVIATION_THRESHOLD: BigDecimal = BigDecimal(10_000)
    }
}
