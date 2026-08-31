package com.gemwallet.android.application.swap.cases

import kotlinx.coroutines.flow.Flow

interface RequestSwapQuotes {
    operator fun invoke(
        requestParams: Flow<SwapQuoteRequestParams?>,
        refreshRequests: Flow<Unit>,
        refreshEnabled: Flow<Boolean>,
        onFetchStarted: (SwapQuoteRequestKey) -> Unit,
        refreshIntervalMillis: Long,
    ): Flow<SwapQuotesResult?>

    companion object {
        const val QUOTE_DEBOUNCE_MS = 500L
    }
}
