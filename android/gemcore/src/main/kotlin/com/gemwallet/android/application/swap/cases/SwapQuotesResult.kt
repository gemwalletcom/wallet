package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.GemSwapQuotesResult
import uniffi.gemstone.GemSwapRequest
import uniffi.gemstone.SwapperException
import uniffi.gemstone.SwapperQuote

data class SwapQuotesResult(val items: List<SwapperQuote> = emptyList(), val requestKey: GemSwapRequest, val pay: AssetInfo, val receive: AssetInfo, val err: Throwable? = null)

fun SwapQuotesResult.toGem(): GemSwapQuotesResult = GemSwapQuotesResult(
    request = requestKey,
    quotes = items,
    error = err?.let { it as? SwapperException ?: SwapperException.ComputeQuoteException(it.message.orEmpty()) },
)
