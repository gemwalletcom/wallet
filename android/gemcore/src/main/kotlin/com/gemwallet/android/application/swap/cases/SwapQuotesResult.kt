package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.SwapperQuote
import uniffi.gemstone.GemSwapQuotesResult
import uniffi.gemstone.SwapperException

data class SwapQuotesResult(
    val items: List<SwapperQuote> = emptyList(),
    val requestKey: SwapQuoteRequestKey,
    val pay: AssetInfo,
    val receive: AssetInfo,
    val err: Throwable? = null,
)

fun SwapQuotesResult.matches(params: SwapQuoteRequestParams?): Boolean =
    params?.key == requestKey

fun SwapQuotesResult.toGem(): GemSwapQuotesResult = GemSwapQuotesResult(
    request = requestKey.toGem(),
    quotes = items,
    error = err?.let { it as? SwapperException ?: SwapperException.ComputeQuoteException(it.message.orEmpty()) },
)
