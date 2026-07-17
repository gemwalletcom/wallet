package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.NumberSanitizer
import com.gemwallet.android.math.parseInputNumberOrNull
import java.math.BigDecimal

object SwapSlippage {
    val defaultBps: UInt = 100u
    val suggestionsBps: List<UInt> = listOf(30u, 50u, 300u)
    const val maxPercent: Int = 20
    private val minPercent: BigDecimal = BigDecimal("0.1")

    val maxPercentLabel: String = "$maxPercent%"
    val minPercentLabel: String = "${minPercent.toPlainString()}%"

    fun format(bps: UInt): String =
        bps.toLong().toBigDecimal().movePointLeft(2).stripTrailingZeros().toPlainString()

    fun sanitize(input: String): String =
        NumberSanitizer(maximumFractionDigits = 2, maximumIntegerDigits = 2).sanitize(input)

    fun parseBps(input: String): UInt? {
        val percent = input.parseInputNumberOrNull()?.takeIf { it > BigDecimal.ZERO } ?: return null
        return (percent.min(BigDecimal(maxPercent)) * BigDecimal(100)).toInt().toUInt()
    }

    fun isOverMax(input: String): Boolean =
        (input.parseInputNumberOrNull() ?: BigDecimal.ZERO) > BigDecimal(maxPercent)

    fun isBelowMin(input: String): Boolean {
        val percent = input.parseInputNumberOrNull() ?: return false
        return percent > BigDecimal.ZERO && percent < minPercent
    }
}
