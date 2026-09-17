package com.gemwallet.android.ui.models.swap

import com.gemwallet.android.math.parseInputNumberOrNull
import uniffi.gemstone.GemNumberFormat
import java.text.DecimalFormatSymbols

object SwapSlippage {

    fun percentLabel(bps: UInt, slippageText: (UInt) -> String): String = "${slippageText(bps)}%"

    fun numberFormat(): GemNumberFormat = GemNumberFormat(DecimalFormatSymbols.getInstance().decimalSeparator.toString())

    fun sanitize(input: String, maximumFractionDigits: UInt, maximumIntegerDigits: UInt): String =
        numberFormat().sanitize(input, maximumFractionDigits, maximumIntegerDigits)

    fun parseBps(input: String, slippageBps: (Double) -> UInt?): UInt? {
        val percent = input.parseInputNumberOrNull() ?: return null
        return slippageBps(percent.toDouble())
    }
}
