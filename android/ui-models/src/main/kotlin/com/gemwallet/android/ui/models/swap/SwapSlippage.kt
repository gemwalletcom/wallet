package com.gemwallet.android.ui.models.swap

import uniffi.gemstone.GemNumberFormat
import java.text.DecimalFormatSymbols

object SwapSlippage {

    fun numberFormat(): GemNumberFormat = GemNumberFormat(DecimalFormatSymbols.getInstance().decimalSeparator.toString())
}
