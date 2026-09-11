package com.gemwallet.android.domains.swap

import uniffi.gemstone.GemAssetRate
import uniffi.gemstone.GemSwapRate
import java.math.BigDecimal
import java.math.MathContext
import java.math.RoundingMode
import java.text.NumberFormat
import java.util.Locale

data class AssetRatePair(
    val forward: String,
    val reverse: String,
)

class AssetRateFormatter(
    private val locale: Locale = Locale.getDefault(),
) {
    fun format(rate: GemSwapRate): AssetRatePair = AssetRatePair(forward = format(rate.direct), reverse = format(rate.inverse))

    fun format(rate: GemAssetRate): String = "1 ${rate.baseSymbol} ≈ ${formatAmount(BigDecimal.valueOf(rate.value), rate.quoteSymbol)}"

    private fun formatAmount(value: BigDecimal, symbol: String): String {
        val formatter = NumberFormat.getNumberInstance(locale).apply {
            isGroupingUsed = true
        }
        val formattedValue = when {
            value.abs() >= NORMAL_FORMAT_THRESHOLD || value.compareTo(BigDecimal.ZERO) == 0 || value.abs() < MIN_RATE_THRESHOLD -> {
                formatter.minimumFractionDigits = DEFAULT_FRACTION_DIGITS
                formatter.maximumFractionDigits = DEFAULT_FRACTION_DIGITS
                value
            }
            else -> {
                formatter.minimumFractionDigits = 0
                formatter.maximumFractionDigits = SMALL_VALUE_MAX_FRACTION_DIGITS
                value.round(MathContext(SMALL_VALUE_SIGNIFICANT_DIGITS, RoundingMode.HALF_UP))
            }
        }

        return "${formatter.format(formattedValue)} $symbol"
    }

    private companion object {
        val NORMAL_FORMAT_THRESHOLD: BigDecimal = BigDecimal("0.99")
        val MIN_RATE_THRESHOLD: BigDecimal = BigDecimal("0.0000000001")
        const val DEFAULT_FRACTION_DIGITS = 2
        const val SMALL_VALUE_MAX_FRACTION_DIGITS = 8
        const val SMALL_VALUE_SIGNIFICANT_DIGITS = 4
    }
}
