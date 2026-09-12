package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.parseInputNumberOrNull
import uniffi.gemstone.Config
import uniffi.gemstone.GemNumberSanitizer
import java.text.DecimalFormatSymbols
import java.math.BigDecimal
import com.gemwallet.android.domains.gemConfig

object SwapSlippage {
    private val config by lazy { gemConfig.getSwapConfig() }
    val suggestionsBps: List<UInt> = config.slippageSuggestionsBps
    val maxBps: UInt = config.maxSlippageBps
    private val minBps: UInt = config.minSlippageBps

    fun maxPercentLabel(slippagePercent: (UInt) -> Double): String = "${format(maxBps, slippagePercent)}%"

    fun minPercentLabel(slippagePercent: (UInt) -> Double): String = "${format(minBps, slippagePercent)}%"

    fun format(bps: UInt, slippagePercent: (UInt) -> Double): String =
        slippagePercent(bps).toBigDecimal().stripTrailingZeros().toPlainString()

    fun sanitize(input: String): String =
        GemNumberSanitizer(DecimalFormatSymbols.getInstance().decimalSeparator.toString(), 2u, 2u).sanitize(input)

    fun parseBps(input: String, slippageBps: (Double) -> UInt?): UInt? {
        val percent = input.parseInputNumberOrNull() ?: return null
        return slippageBps(percent.toDouble())
    }
}
