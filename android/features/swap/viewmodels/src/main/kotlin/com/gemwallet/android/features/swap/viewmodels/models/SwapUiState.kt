package com.gemwallet.android.features.swap.viewmodels.models

import com.gemwallet.android.ui.models.ButtonState
import uniffi.gemstone.GemSwapButtonAction
import uniffi.gemstone.GemSwapButtonState
import uniffi.gemstone.GemSwapSession
import uniffi.gemstone.GemSwapSessionAction
import uniffi.gemstone.SwapperException

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
    val action: GemSwapSessionAction = GemSwapSessionAction.None,
    val buttonAction: GemSwapButtonAction = GemSwapButtonAction.Swap,
    val buttonState: ButtonState = ButtonState.Disabled,
    val error: SwapperException? = null,
    val isQuoteLoading: Boolean = false,
    val isTransferLoading: Boolean = false,
    val isInputEmpty: Boolean = true,
) {
    val isReceiveLoading: Boolean
        get() = isQuoteLoading && !isTransferLoading

    val isQuoteInteractionEnabled: Boolean
        get() = !isTransferLoading

    val payItemInteraction: SwapItemInteraction
        get() = SwapItemInteraction.pay(isQuoteInteractionEnabled)

    val receiveItemInteraction: SwapItemInteraction
        get() = SwapItemInteraction.receive(isQuoteInteractionEnabled)
}

internal fun createSwapUiState(session: GemSwapSession, buttonAction: GemSwapButtonAction) = SwapUiState(
    action = session.action(),
    buttonAction = buttonAction,
    buttonState = when (session.buttonState(buttonAction)) {
        GemSwapButtonState.DISABLED -> ButtonState.Disabled
        GemSwapButtonState.LOADING -> ButtonState.Loading
        GemSwapButtonState.ENABLED -> ButtonState.Enabled
    },
    error = session.error(),
    isQuoteLoading = session.isQuoteLoading(),
    isTransferLoading = session.isTransferLoading(),
    isInputEmpty = session.isInputEmpty(),
)
