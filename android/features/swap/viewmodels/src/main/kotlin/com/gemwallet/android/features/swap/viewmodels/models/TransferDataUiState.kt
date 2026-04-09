package com.gemwallet.android.features.swap.viewmodels.models

import uniffi.gemstone.SwapperProvider

internal sealed interface TransferDataUiState {
    data object Idle : TransferDataUiState
    data class Loading(
        val quoteKey: QuoteRequestKey,
        val providerId: SwapperProvider,
    ) : TransferDataUiState

    data class Error(
        val quoteKey: QuoteRequestKey,
        val providerId: SwapperProvider,
        val error: SwapError,
    ) : TransferDataUiState
}

internal val TransferDataUiState.isLoading: Boolean
    get() = this is TransferDataUiState.Loading

internal fun TransferDataUiState.matches(snapshot: TransferQuoteSnapshot): Boolean = when (this) {
    TransferDataUiState.Idle -> false
    is TransferDataUiState.Loading -> quoteKey == snapshot.requestKey && providerId == snapshot.providerId
    is TransferDataUiState.Error -> quoteKey == snapshot.requestKey && providerId == snapshot.providerId
}
