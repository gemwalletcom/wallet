package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.ui.models.ButtonState
import uniffi.gemstone.GemSwapButtonAction

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
    val buttonAction: GemSwapButtonAction = GemSwapButtonAction.Swap,
    val hasQuote: Boolean = false,
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
        get() = when {
            buttonAction is GemSwapButtonAction.InsufficientBalance -> ButtonState.Disabled
            isQuoteLoading || isTransferLoading -> ButtonState.Loading
            buttonAction is GemSwapButtonAction.Swap && !hasQuote -> ButtonState.Disabled
            else -> ButtonState.Enabled
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

internal fun createSwapUiState(session: SwapQuoteSession, buttonAction: GemSwapButtonAction): SwapUiState {
    val quotePhase = session.quotePhase
    val transferPhase = session.transferPhase

    val action = when {
        transferPhase is SwapTransferPhase.Loading -> SwapActionState.TransferLoading
        transferPhase is SwapTransferPhase.Failed -> SwapActionState.TransferError(SwapError.toError(transferPhase.error))
        quotePhase is SwapQuotePhase.Loading -> SwapActionState.QuoteLoading
        quotePhase is SwapQuotePhase.Failed -> SwapActionState.QuoteError(SwapError.toError(quotePhase.error))
        session.quote != null -> SwapActionState.Ready
        else -> SwapActionState.None
    }

    return SwapUiState(
        action = action,
        buttonAction = buttonAction,
        hasQuote = session.quote != null,
        isQuoteLoading = quotePhase is SwapQuotePhase.Loading,
        isTransferLoading = transferPhase is SwapTransferPhase.Loading,
        isInputEmpty = quotePhase is SwapQuotePhase.NoInput,
    )
}
