package com.gemwallet.android.features.swap.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.features.swap.viewmodels.localization.stringRes
import com.gemwallet.android.features.swap.viewmodels.localization.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.list_item.infoSheet
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import uniffi.gemstone.GemSwapSessionAction
import uniffi.gemstone.GemSwapSideInteraction
import uniffi.gemstone.GemSwapViewState

data class SwapUiState(
    @StringRes val actionTitle: Int = R.string.wallet_swap,
    val buttonState: ButtonState = ButtonState.Disabled,
    val errorText: String? = null,
    val errorInfo: InfoSheetEntity? = null,
    val isQuoteLoading: Boolean = false,
    val isTransferLoading: Boolean = false,
    val isInputEmpty: Boolean = true,
    val payItemInteraction: GemSwapSideInteraction = GemSwapSideInteraction(isAmountEditable = true, isAssetSelectable = true, isBalanceActionEnabled = true),
    val receiveItemInteraction: GemSwapSideInteraction = GemSwapSideInteraction(isAmountEditable = false, isAssetSelectable = true, isBalanceActionEnabled = false),
    val isReceiveLoading: Boolean = false,
) {
    val isQuoteInteractionEnabled: Boolean
        get() = !isTransferLoading
}

internal fun createSwapUiState(state: GemSwapViewState, context: Context) = SwapUiState(
    actionTitle = state.buttonAction.stringRes(),
    buttonState = state.buttonState.buttonState(),
    errorText = state.error?.text(context),
    errorInfo = state.error?.info()?.infoSheet(context, null),
    isQuoteLoading = state.isQuoteLoading,
    isTransferLoading = state.isTransferLoading,
    isInputEmpty = state.isInputEmpty,
    payItemInteraction = state.pay,
    receiveItemInteraction = state.receive,
    isReceiveLoading = state.isReceiveLoading,
)
