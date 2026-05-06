package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.application.swap.coordinators.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.coordinators.SwapQuotesResult

internal sealed interface QuoteUiState {
    data object NoInput : QuoteUiState
    data class Loading(val requestKey: SwapQuoteRequestKey) : QuoteUiState
    data class Ready(val quotes: SwapQuotesResult) : QuoteUiState
    data class Error(val requestKey: SwapQuoteRequestKey, val error: SwapError) : QuoteUiState
}

internal val QuoteUiState.requestKey: SwapQuoteRequestKey?
    get() = when (this) {
        QuoteUiState.NoInput -> null
        is QuoteUiState.Loading -> requestKey
        is QuoteUiState.Ready -> quotes.requestKey
        is QuoteUiState.Error -> requestKey
    }
