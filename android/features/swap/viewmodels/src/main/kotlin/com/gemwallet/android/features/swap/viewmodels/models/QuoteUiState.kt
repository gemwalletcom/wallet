package com.gemwallet.android.features.swap.viewmodels.models

internal sealed interface QuoteUiState {
    data object NoInput : QuoteUiState
    data class Loading(val requestKey: QuoteRequestKey) : QuoteUiState
    data class Ready(val quotes: QuotesState) : QuoteUiState
    data class Error(val requestKey: QuoteRequestKey, val error: SwapError) : QuoteUiState
}

internal val QuoteUiState.requestKey: QuoteRequestKey?
    get() = when (this) {
        QuoteUiState.NoInput -> null
        is QuoteUiState.Loading -> requestKey
        is QuoteUiState.Ready -> quotes.requestKey
        is QuoteUiState.Error -> requestKey
    }
