package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.NumberSanitizer
import com.gemwallet.android.math.parseNumberOrNull
import java.math.BigDecimal

object SwapSlippage {
    val defaultBps: UInt = 100u
    const val maxPercent: Int = 20

    fun format(bps: UInt): String =
        bps.toLong().toBigDecimal().movePointLeft(2).stripTrailingZeros().toPlainString()

    fun sanitize(input: String): String =
        NumberSanitizer(maximumFractionDigits = 2, maximumIntegerDigits = 2).sanitize(input)

    fun parseBps(input: String): UInt? {
        val percent = input.parseNumberOrNull()?.takeIf { it > BigDecimal.ZERO } ?: return null
        return (percent.min(BigDecimal(maxPercent)) * BigDecimal(100)).toInt().toUInt()
    }

    fun isOverMax(input: String): Boolean =
        (input.parseNumberOrNull() ?: BigDecimal.ZERO) > BigDecimal(maxPercent)
}
