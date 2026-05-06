package com.gemwallet.android.application.swap.coordinators

import com.gemwallet.android.model.AssetInfo
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.SwapperQuote

data class SwapQuotesResult(
    val items: List<SwapperQuote> = emptyList(),
    val requestKey: SwapQuoteRequestKey,
    val pay: AssetInfo,
    val receive: AssetInfo,
    val err: Throwable? = null,
)

fun SwapQuotesResult.getQuote(provider: SwapperProvider?): SwapperQuote? =
    items.firstOrNull { it.data.provider.id == provider } ?: items.firstOrNull()

fun SwapQuotesResult.matches(params: SwapQuoteRequestParams?): Boolean =
    params?.key == requestKey
