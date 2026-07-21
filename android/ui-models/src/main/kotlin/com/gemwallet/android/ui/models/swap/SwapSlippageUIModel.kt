package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.NumberSanitizer
import com.gemwallet.android.math.parseInputNumberOrNull
import java.math.BigDecimal

data class SwapSlippageUIModel(
    val defaultBps: UInt,
    val suggestionsBps: List<UInt>,
    val minBps: UInt,
    val maxBps: UInt,
    val highWarningBps: UInt,
) {
    private val minPercent: BigDecimal = bpsToPercent(minBps)
    private val maxPercent: BigDecimal = bpsToPercent(maxBps)

    val minPercentLabel: String = "${format(minBps)}%"
    val maxPercentLabel: String = "${format(maxBps)}%"

    fun sanitize(input: String): String =
        NumberSanitizer(
            maximumFractionDigits = 2,
            maximumIntegerDigits = maxPercent.toBigInteger().toString().length,
        ).sanitize(input)

    fun parseBps(input: String): UInt? {
        val percent = input.parseInputNumberOrNull()?.takeIf { it > BigDecimal.ZERO } ?: return null
        return (percent.min(maxPercent) * BigDecimal(100)).toInt().toUInt()
    }

    fun isOverMax(input: String): Boolean =
        (input.parseInputNumberOrNull() ?: BigDecimal.ZERO) > maxPercent

    fun isBelowMin(input: String): Boolean {
        val percent = input.parseInputNumberOrNull() ?: return false
        return percent > BigDecimal.ZERO && percent < minPercent
    }

    companion object {
        fun format(bps: UInt): String =
            bpsToPercent(bps).stripTrailingZeros().toPlainString()

        private fun bpsToPercent(bps: UInt): BigDecimal =
            bps.toLong().toBigDecimal().movePointLeft(2)
    }
}
