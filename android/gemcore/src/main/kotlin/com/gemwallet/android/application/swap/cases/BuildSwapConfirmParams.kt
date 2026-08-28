package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import uniffi.gemstone.SwapperQuote

interface BuildSwapConfirmParams {
    suspend operator fun invoke(
        quote: SwapperQuote,
        pay: AssetInfo,
        receive: AssetInfo,
    ): ConfirmParams.SwapParams?
}

class SwapNoQuoteException(cause: Throwable? = null) : Exception(cause)
