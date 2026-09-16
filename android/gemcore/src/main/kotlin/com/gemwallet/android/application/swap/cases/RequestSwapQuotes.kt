package com.gemwallet.android.application.swap.cases

import kotlinx.coroutines.flow.Flow
import uniffi.gemstone.GemSwapRequest

interface RequestSwapQuotes {
    operator fun invoke(
        requestParams: Flow<SwapQuoteRequestParams?>,
        refreshRequests: Flow<Unit>,
        refreshEnabled: Flow<Boolean>,
        onFetchStarted: (GemSwapRequest) -> Unit,
        refreshIntervalMillis: Long,
        debounceMillis: Long,
    ): Flow<SwapQuotesResult?>
}
