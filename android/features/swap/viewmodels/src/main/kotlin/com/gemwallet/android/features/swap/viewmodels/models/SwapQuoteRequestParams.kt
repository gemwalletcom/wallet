package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.GemSwapQuoteInput
import uniffi.gemstone.GemSwapRequest

internal data class SwapQuoteRequestParams(val input: GemSwapQuoteInput, val pay: AssetInfo, val receive: AssetInfo) {
    val key: GemSwapRequest
        get() = input.request
}
