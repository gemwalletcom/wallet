package com.gemwallet.android.model

import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemNumberDisplay
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.formattedAmount
import java.math.BigDecimal
import java.math.BigInteger
import java.math.RoundingMode
import java.text.DecimalFormat
import java.text.NumberFormat
import java.util.Locale

class ValueFormatter(private val style: GemValueStyle, private val locale: Locale = Locale.getDefault()) {

    fun string(value: BigInteger, asset: Asset): String = string(value, decimals = asset.decimals, currency = asset.symbol)

    fun string(value: BigInteger, decimals: Int, currency: String = ""): String = string(BigDecimal(value).movePointLeft(decimals), currency)

    fun string(value: BigDecimal, currency: String = ""): String {
        if (value.signum() == 0) return appendCurrency("0", currency)

        val number = formattedAmount(value.toDouble(), currency.ifEmpty { null }, style)
        val precision = (number.display as? GemNumberDisplay.Number)?.precision ?: return number.text(locale)

        val formatter = (NumberFormat.getInstance(locale) as DecimalFormat).apply {
            roundingMode = ROUNDING_MODE
        }
        return appendCurrency(formatter.format(value, precision), currency)
    }

    private fun appendCurrency(value: String, currency: String): String = if (currency.isEmpty()) value else "$value $currency"

    companion object {
        private val ROUNDING_MODE: RoundingMode = RoundingMode.DOWN
    }
}
