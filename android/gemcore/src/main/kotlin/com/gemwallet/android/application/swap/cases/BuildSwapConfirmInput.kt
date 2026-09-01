package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.SwapperQuote

interface BuildSwapConfirmInput {
    suspend operator fun invoke(
        quote: SwapperQuote,
        pay: AssetInfo,
        receive: AssetInfo,
    ): GemConfirmInput?
}

class SwapNoQuoteException(cause: Throwable? = null) : Exception(cause)
