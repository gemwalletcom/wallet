package com.gemwallet.android.application.swap.cases

import kotlinx.coroutines.flow.Flow

interface RequestSwapQuotes {
    operator fun invoke(
        requestParams: Flow<SwapQuoteRequestParams?>,
        refreshRequests: Flow<Unit>,
        refreshEnabled: Flow<Boolean>,
        onFetchStarted: (SwapQuoteRequestKey) -> Unit,
        refreshIntervalMillis: Long,
        debounceMillis: Long,
    ): Flow<SwapQuotesResult?>
}
