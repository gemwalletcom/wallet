package com.gemwallet.android.model

import android.icu.text.CompactDecimalFormat
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.dustThreshold
import uniffi.gemstone.dustThresholdPlaces
import java.math.BigDecimal
import java.math.BigInteger
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale

class ValueFormatter(
    private val style: GemValueStyle,
    private val locale: Locale = Locale.getDefault(),
) {

    fun string(value: BigInteger, asset: Asset): String =
        string(value, decimals = asset.decimals, currency = asset.symbol)

    fun string(value: BigInteger, decimals: Int, currency: String = ""): String =
        string(BigDecimal(value).movePointLeft(decimals), currency)

    fun string(value: BigDecimal, currency: String = ""): String {
        if (value.signum() == 0) return appendCurrency("0", currency)

        if (style.abbreviates(value.toDouble())) {
            return appendCurrency(abbreviated(value), currency)
        }

        if (style.isDust(value.toDouble())) {
            return appendCurrency("<${formattedDustThreshold()}", currency)
        }

        val formatter = (NumberFormat.getInstance(locale) as DecimalFormat).apply {
            roundingMode = ROUNDING_MODE
        }
        return appendCurrency(formatter.format(value, precision(value.abs())), currency)
    }

    fun rounded(value: BigDecimal): BigDecimal = value.rounded(precision(value.abs()), ROUNDING_MODE)

    private fun precision(magnitude: BigDecimal): Precision = style.precision(magnitude.toDouble()).toPrecision()

    private fun abbreviated(decimal: BigDecimal): String {
        val formatter = CompactDecimalFormat.getInstance(locale, CompactDecimalFormat.CompactStyle.SHORT)
        formatter.setSignificantDigitsUsed(false)
        formatter.minimumFractionDigits = 0
        formatter.maximumFractionDigits = 2
        formatter.roundingMode = android.icu.math.BigDecimal.ROUND_DOWN
        return formatter.format(decimal)
    }

    private fun formattedDustThreshold(): String {
        val formatter = (NumberFormat.getInstance(locale) as DecimalFormat).apply {
            minimumFractionDigits = dustThresholdPlaces().toInt()
            maximumFractionDigits = dustThresholdPlaces().toInt()
        }
        return formatter.format(dustThreshold())
    }

    private fun appendCurrency(value: String, currency: String): String =
        if (currency.isEmpty()) value else "$value $currency"

    companion object {
        private val ROUNDING_MODE: RoundingMode = RoundingMode.DOWN
    }
}
