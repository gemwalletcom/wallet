package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.features.buy.localization.stringRes
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ButtonState
import uniffi.gemstone.GemFiatButtonAction
import uniffi.gemstone.GemFiatButtonState
import uniffi.gemstone.GemFiatQuotePhase
import uniffi.gemstone.GemFiatViewState

data class FiatUiState(
    val isLoading: Boolean = false,
    val errorText: String? = null,
    @StringRes val actionTitle: Int = R.string.common_continue,
    val retries: Boolean = false,
    val buttonState: ButtonState = ButtonState.Disabled,
    val canSelectProvider: Boolean = false,
)

internal fun createFiatUiState(state: GemFiatViewState, errorText: String?) = FiatUiState(
    isLoading = state.phase is GemFiatQuotePhase.Loading,
    errorText = errorText,
    actionTitle = state.buttonAction.stringRes(),
    retries = state.buttonAction == GemFiatButtonAction.RETRY_QUOTE,
    buttonState = when (state.buttonState) {
        GemFiatButtonState.DISABLED -> ButtonState.Disabled
        GemFiatButtonState.LOADING -> ButtonState.Loading
        GemFiatButtonState.ENABLED -> ButtonState.Enabled
    },
    canSelectProvider = state.canSelectProvider,
)
