package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.GemSwapQuoteInput
import uniffi.gemstone.GemSwapRequest

data class SwapQuoteRequestParams(val input: GemSwapQuoteInput, val pay: AssetInfo, val receive: AssetInfo) {
    val key: GemSwapRequest
        get() = input.request
}
