package com.gemwallet.android.features.swap.viewmodels.models

import com.wallet.core.primitives.AssetData
import uniffi.gemstone.GemSwapQuoteInput
import uniffi.gemstone.GemSwapRequest

internal data class SwapQuoteRequestParams(val input: GemSwapQuoteInput, val pay: AssetData, val receive: AssetData) {
    val key: GemSwapRequest
        get() = input.request
}
