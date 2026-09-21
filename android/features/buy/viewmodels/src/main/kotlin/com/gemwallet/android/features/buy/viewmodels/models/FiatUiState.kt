package com.gemwallet.android.features.buy.viewmodels.models

import androidx.annotation.StringRes
import com.gemwallet.android.features.buy.localization.stringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import uniffi.gemstone.GemFiatButtonAction
import uniffi.gemstone.GemFiatButtonState
import uniffi.gemstone.GemFiatQuotePhase
import uniffi.gemstone.GemFiatViewState

data class FiatUiState(
    val isLoading: Boolean = false,
    val amountError: String? = null,
    val quotesMessage: String? = null,
    @StringRes val actionTitle: Int = R.string.common_continue,
    val retries: Boolean = false,
    val buttonState: ButtonState = ButtonState.Disabled,
    val canSelectProvider: Boolean = false,
)

internal fun createFiatUiState(state: GemFiatViewState, amountError: String?, quotesMessage: String?) = FiatUiState(
    isLoading = state.phase is GemFiatQuotePhase.Loading,
    amountError = amountError,
    quotesMessage = quotesMessage,
    actionTitle = state.buttonAction.stringRes(),
    retries = state.buttonAction == GemFiatButtonAction.RETRY_QUOTE,
    buttonState = when (state.buttonState) {
        GemFiatButtonState.DISABLED -> ButtonState.Disabled
        GemFiatButtonState.LOADING -> ButtonState.Loading
        GemFiatButtonState.ENABLED -> ButtonState.Enabled
    },
    canSelectProvider = state.canSelectProvider,
)
