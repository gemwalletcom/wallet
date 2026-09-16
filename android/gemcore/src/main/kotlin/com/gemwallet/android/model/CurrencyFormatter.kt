package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import com.wallet.core.primitives.Currency
import java.math.BigDecimal
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale
import uniffi.gemstone.GemCurrencyStyle
import uniffi.gemstone.GemPrecision

class CurrencyFormatter(
    private val type: Type = Type.Currency,
    private val currencyCode: String,
    private val locale: Locale = Locale.getDefault(),
) {
    constructor(
        type: Type = Type.Currency,
        currency: Currency,
        locale: Locale = Locale.getDefault(),
    ) : this(type, currency.string, locale)

    enum class Type { Currency, Fiat, Abbreviated }

    private val currencyFormatter: DecimalFormat by lazy {
        (NumberFormat.getCurrencyInstance(locale) as DecimalFormat).apply {
            currency = java.util.Currency.getInstance(currencyCode)
            roundingMode = RoundingMode.HALF_EVEN
        }
    }

    private val abbreviatedFormatter: CompactDecimalFormat by lazy {
        CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT).apply {
            currency = android.icu.util.Currency.getInstance(currencyCode)
            setSignificantDigitsUsed(false)
            minimumFractionDigits = 0
            maximumFractionDigits = 2
            roundingMode = android.icu.math.BigDecimal.ROUND_DOWN
        }
    }

    fun string(value: Double): String = string(BigDecimal.valueOf(value))

    fun string(value: BigDecimal): String =
        if (style.abbreviates(value.abs().toDouble())) {
            abbreviatedFormatter.format(value)
        } else {
            currencyFormatter.format(value, precision(value.abs()))
        }

    private val style: GemCurrencyStyle
        get() = when (type) {
            Type.Currency -> GemCurrencyStyle.CURRENCY
            Type.Fiat -> GemCurrencyStyle.FIAT
            Type.Abbreviated -> GemCurrencyStyle.ABBREVIATED
        }

    private fun precision(magnitude: BigDecimal): Precision = when (val precision = style.precision(magnitude.toDouble())) {
        is GemPrecision.Fraction -> Precision.Fraction(min = precision.min.toInt(), max = precision.max.toInt())
        is GemPrecision.Significant -> Precision.Significant(max = precision.max.toInt())
    }
}
