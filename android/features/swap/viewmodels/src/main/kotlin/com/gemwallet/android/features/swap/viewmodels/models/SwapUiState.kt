package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState

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
    val isInputEmpty: Boolean = true,
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

    val buttonState: ButtonState
        get() = when (val currentAction = action) {
            SwapActionState.Ready,
            is SwapActionState.TransferError -> ButtonState.Enabled
            is SwapActionState.QuoteError -> buttonState(enabled = currentAction.error !is SwapError.InsufficientBalance)
            SwapActionState.QuoteLoading,
            SwapActionState.TransferLoading -> ButtonState.Loading
            SwapActionState.None -> ButtonState.Disabled
        }

    val isReceiveLoading: Boolean
        get() = isQuoteLoading && !isTransferLoading

    val isQuoteInteractionEnabled: Boolean
        get() = !isTransferLoading

    val payItemInteraction: SwapItemInteraction
        get() = SwapItemInteraction.pay(isQuoteInteractionEnabled)

    val receiveItemInteraction: SwapItemInteraction
        get() = SwapItemInteraction.receive(isQuoteInteractionEnabled)
}

internal fun createSwapUiState(session: SwapQuoteSession): SwapUiState {
    val quotePhase = session.quotePhase
    val transferPhase = session.transferPhase
    val displayedQuote = session.quote
    val validationError = displayedQuote?.validationError

    val action = when {
        transferPhase is SwapTransferPhase.Loading -> SwapActionState.TransferLoading
        transferPhase is SwapTransferPhase.Failed -> SwapActionState.TransferError(transferPhase.error)
        quotePhase is SwapQuotePhase.Loading -> SwapActionState.QuoteLoading
        quotePhase is SwapQuotePhase.Failed -> SwapActionState.QuoteError(quotePhase.error)
        validationError != null -> SwapActionState.QuoteError(validationError)
        displayedQuote != null -> SwapActionState.Ready
        else -> SwapActionState.None
    }

    return SwapUiState(
        action = action,
        isQuoteLoading = quotePhase is SwapQuotePhase.Loading,
        isTransferLoading = transferPhase is SwapTransferPhase.Loading,
        isInputEmpty = quotePhase is SwapQuotePhase.NoInput,
    )
}
