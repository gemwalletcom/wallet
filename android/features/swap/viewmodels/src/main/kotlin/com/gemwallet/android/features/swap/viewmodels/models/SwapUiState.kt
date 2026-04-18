package com.gemwallet.android.features.swap.viewmodels.models

import uniffi.gemstone.SwapperProvider

sealed interface SwapActionState {
    data object None : SwapActionState
    data object QuoteLoading : SwapActionState
    data object Ready : SwapActionState
    data object TransferLoading : SwapActionState
    data class QuoteError(val error: SwapError) : SwapActionState
    data class TransferError(val error: SwapError) : SwapActionState
}

data class SwapItemInteraction(
    val isAmountEditable: Boolean,
    val isAssetSelectable: Boolean,
    val isBalanceActionEnabled: Boolean,
) {
    companion object {
        fun pay(isEnabled: Boolean) = SwapItemInteraction(
            isAmountEditable = isEnabled,
            isAssetSelectable = isEnabled,
            isBalanceActionEnabled = isEnabled,
        )

        fun receive(isEnabled: Boolean) = SwapItemInteraction(
            isAmountEditable = false,
            isAssetSelectable = isEnabled,
            isBalanceActionEnabled = false,
        )
    }
}

data class SwapUiState(
    val action: SwapActionState = SwapActionState.None,
    val isQuoteLoading: Boolean = false,
    val isTransferLoading: Boolean = false,
    val isSwapButtonVisible: Boolean = false,
) {
    val error: SwapError?
        get() = when (val currentAction = action) {
            is SwapActionState.QuoteError -> currentAction.error
            is SwapActionState.TransferError -> currentAction.error
            SwapActionState.None,
            SwapActionState.QuoteLoading,
            SwapActionState.Ready,
            SwapActionState.TransferLoading -> null
        }

    val isReceiveLoading: Boolean
        get() = isQuoteLoading

    val isQuoteInteractionEnabled: Boolean
        get() = !isTransferLoading

    val payItemInteraction: SwapItemInteraction
        get() = SwapItemInteraction.pay(isQuoteInteractionEnabled)

    val receiveItemInteraction: SwapItemInteraction
        get() = SwapItemInteraction.receive(isQuoteInteractionEnabled)
}

internal data class TransferQuoteSnapshot(
    val quotes: QuotesState,
    val selectedProvider: SwapperProvider?,
    val quote: QuoteState,
) {
    val requestKey: QuoteRequestKey
        get() = quotes.requestKey

    val providerId: SwapperProvider
        get() = quote.quote.data.provider.id

    companion object
}

internal fun TransferQuoteSnapshot.Companion.create(
    quotes: QuotesState,
    selectedProvider: SwapperProvider?,
): TransferQuoteSnapshot? {
    val quote = quotes.getQuote(selectedProvider)?.let { QuoteState(it, quotes.pay, quotes.receive) } ?: return null
    return TransferQuoteSnapshot(
        quotes = quotes,
        selectedProvider = selectedProvider,
        quote = quote,
    )
}

internal fun createSwapUiState(
    quoteState: QuoteUiState,
    transferState: TransferDataUiState,
    displayedQuote: QuoteState?,
): SwapUiState {
    val action = when {
        transferState is TransferDataUiState.Loading -> SwapActionState.TransferLoading
        transferState is TransferDataUiState.Error -> SwapActionState.TransferError(transferState.error)
        quoteState is QuoteUiState.Loading -> SwapActionState.QuoteLoading
        quoteState is QuoteUiState.Error -> SwapActionState.QuoteError(quoteState.error)
        displayedQuote?.validationError != null -> SwapActionState.QuoteError(displayedQuote.validationError!!)
        displayedQuote != null -> SwapActionState.Ready
        else -> SwapActionState.None
    }

    return SwapUiState(
        action = action,
        isQuoteLoading = quoteState is QuoteUiState.Loading,
        isTransferLoading = transferState is TransferDataUiState.Loading,
        isSwapButtonVisible = quoteState !is QuoteUiState.NoInput,
    )
}
