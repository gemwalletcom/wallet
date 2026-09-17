package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.parseInputNumberOrNull
import uniffi.gemstone.GemNumberFormat
import java.text.DecimalFormatSymbols

object SwapSlippage {

    fun percentLabel(bps: UInt, slippagePercent: (UInt) -> Double): String = "${format(bps, slippagePercent)}%"

    fun format(bps: UInt, slippagePercent: (UInt) -> Double): String =
        slippagePercent(bps).toBigDecimal().stripTrailingZeros().toPlainString()

    fun sanitize(input: String, maximumFractionDigits: UInt, maximumIntegerDigits: UInt): String =
        GemNumberFormat(DecimalFormatSymbols.getInstance().decimalSeparator.toString())
            .sanitize(input, maximumFractionDigits, maximumIntegerDigits)

    fun parseBps(input: String, slippageBps: (Double) -> UInt?): UInt? {
        val percent = input.parseInputNumberOrNull() ?: return null
        return slippageBps(percent.toDouble())
    }
}
