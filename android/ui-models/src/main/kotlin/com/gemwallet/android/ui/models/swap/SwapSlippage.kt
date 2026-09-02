package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.NumberSanitizer
import com.gemwallet.android.math.parseInputNumberOrNull
import uniffi.gemstone.Config
import java.math.BigDecimal

object SwapSlippage {
    private val config = Config().getSwapConfig()
    val suggestionsBps: List<UInt> = config.slippageSuggestionsBps
    val maxPercent: Int = (config.maxSlippageBps / 100u).toInt()
    private val minPercent: BigDecimal = config.minSlippageBps.toLong().toBigDecimal().movePointLeft(2).stripTrailingZeros()

    val maxPercentLabel: String = "$maxPercent%"
    val minPercentLabel: String = "${minPercent.toPlainString()}%"

    fun format(bps: UInt): String =
        bps.toLong().toBigDecimal().movePointLeft(2).stripTrailingZeros().toPlainString()

    fun sanitize(input: String): String =
        NumberSanitizer(maximumFractionDigits = 2, maximumIntegerDigits = 2).sanitize(input)

    fun parseBps(input: String): UInt? {
        val percent = input.parseInputNumberOrNull()?.takeIf { it > BigDecimal.ZERO } ?: return null
        return (percent * BigDecimal(100)).toInt().toUInt()
    }
}
