package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.parseInputNumberOrNull
import uniffi.gemstone.GemNumberFormat
import uniffi.gemstone.GemSlippageViewState
import java.text.DecimalFormatSymbols

object SwapSlippage {

    fun percentLabel(bps: UInt, slippagePercent: (UInt) -> Double): String = "${format(bps, slippagePercent)}%"

    fun format(bps: UInt, slippagePercent: (UInt) -> Double): String =
        slippagePercent(bps).toBigDecimal().stripTrailingZeros().toPlainString()

    fun sanitize(input: String, state: GemSlippageViewState): String =
        GemNumberFormat(DecimalFormatSymbols.getInstance().decimalSeparator.toString())
            .sanitize(input, state.maximumFractionDigits, state.maximumIntegerDigits)

    fun parseBps(input: String, slippageBps: (Double) -> UInt?): UInt? {
        val percent = input.parseInputNumberOrNull() ?: return null
        return slippageBps(percent.toDouble())
    }
}
