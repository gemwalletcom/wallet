package com.gemwallet.android.features.swap.viewmodels.models

import android.content.Context
import androidx.annotation.StringRes
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSwapSessionAction
import uniffi.gemstone.GemSwapSideInteraction
import uniffi.gemstone.GemSwapViewState

data class SwapUIState(
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
    val payBalance: GemLocalizedText? = null,
    val receiveBalance: GemLocalizedText? = null,
    val payIcon: GemAssetIcon? = null,
    val receiveIcon: GemAssetIcon? = null,
    val payEquivalent: String = "",
    val receiveEquivalent: String = "",
) {
    val isQuoteInteractionEnabled: Boolean
        get() = !isTransferLoading
}

internal fun createSwapUIState(state: GemSwapViewState, context: Context) = SwapUIState(
    actionTitle = state.buttonAction.stringRes(),
    buttonState = state.buttonState.buttonState(),
    errorText = state.error?.text(context),
    errorInfo = state.error?.info()?.infoSheet(),
    isQuoteLoading = state.isQuoteLoading,
    isTransferLoading = state.isTransferLoading,
    isInputEmpty = state.isInputEmpty,
    payItemInteraction = state.pay.interaction,
    receiveItemInteraction = state.receive.interaction,
    isReceiveLoading = state.isReceiveLoading,
    payBalance = state.pay.balance,
    receiveBalance = state.receive.balance,
    payIcon = state.pay.icon,
    receiveIcon = state.receive.icon,
    payEquivalent = state.pay.fiat?.text().orEmpty(),
    receiveEquivalent = state.receive.fiat?.text().orEmpty(),
)
