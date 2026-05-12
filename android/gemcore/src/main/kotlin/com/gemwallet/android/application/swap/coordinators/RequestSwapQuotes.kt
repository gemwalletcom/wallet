package com.gemwallet.android.application.swap.coordinators

import kotlinx.coroutines.flow.Flow

interface RequestSwapQuotes {
    operator fun invoke(
        requestParams: Flow<SwapQuoteRequestParams?>,
        refreshRequests: Flow<Unit>,
        refreshEnabled: Flow<Boolean>,
        onFetchStarted: (SwapQuoteRequestKey) -> Unit,
        refreshIntervalMillis: Long = QUOTE_REFRESH_INTERVAL_MS,
    ): Flow<SwapQuotesResult?>

    companion object {
        const val QUOTE_REFRESH_INTERVAL_MS = 30_000L
        const val QUOTE_DEBOUNCE_MS = 500L
    }
}
