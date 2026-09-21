package com.gemwallet.android.testkit

import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import uniffi.gemstone.SwapperException
import uniffi.gemstone.SwapperQuote

fun mockSwapQuotesResult(params: SwapQuoteRequestParams = mockSwapQuoteRequestParams(), items: List<SwapperQuote> = emptyList(), err: SwapperException? = null) = SwapQuotesResult(
    items = items,
    requestKey = params.key,
    pay = params.pay,
    receive = params.receive,
    err = err,
)
