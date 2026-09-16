package com.gemwallet.android.features.swap.viewmodels.models

import android.content.Context
import com.gemwallet.android.features.swap.viewmodels.localization.text
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.features.swap.viewmodels.localization.stringRes
import com.gemwallet.android.ui.models.ButtonState
import uniffi.gemstone.GemSwapErrorDisplay
import uniffi.gemstone.GemSwapButtonState
import uniffi.gemstone.GemSwapViewState
import uniffi.gemstone.GemSwapSessionAction
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R

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
    @StringRes val actionTitle: Int = R.string.wallet_swap,
    val buttonState: ButtonState = ButtonState.Disabled,
    val errorText: String? = null,
    val errorInfo: InfoSheetEntity? = null,
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

internal fun createSwapUiState(state: GemSwapViewState, context: Context) = SwapUiState(
    actionTitle = state.buttonAction.stringRes(),
    buttonState = when (state.buttonState) {
        GemSwapButtonState.DISABLED -> ButtonState.Disabled
        GemSwapButtonState.LOADING -> ButtonState.Loading
        GemSwapButtonState.ENABLED -> ButtonState.Enabled
    },
    errorText = state.error?.text(context),
    errorInfo = state.error?.infoSheet(),
    isQuoteLoading = state.isQuoteLoading,
    isTransferLoading = state.isTransferLoading,
    isInputEmpty = state.isInputEmpty,
)

private fun GemSwapErrorDisplay.infoSheet(): InfoSheetEntity? = when (this) {
    is GemSwapErrorDisplay.NoQuote -> InfoSheetEntity.NoQuoteInfo
    is GemSwapErrorDisplay.NotSupportedAsset,
    is GemSwapErrorDisplay.MinimumAmount,
    is GemSwapErrorDisplay.AmountTooSmall -> null
}
