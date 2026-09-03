package com.gemwallet.android.application.swap.cases

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.SwapperQuote

data class SwapQuotesResult(
    val items: List<SwapperQuote> = emptyList(),
    val requestKey: SwapQuoteRequestKey,
    val pay: AssetInfo,
    val receive: AssetInfo,
    val err: Throwable? = null,
)

fun SwapQuotesResult.matches(params: SwapQuoteRequestParams?): Boolean =
    params?.key == requestKey
