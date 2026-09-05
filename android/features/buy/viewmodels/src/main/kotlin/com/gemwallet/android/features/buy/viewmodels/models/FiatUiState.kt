package com.gemwallet.android.features.buy.viewmodels.models

import com.gemwallet.android.ui.models.ButtonState
import uniffi.gemstone.GemFiatAmountCheck
import uniffi.gemstone.GemFiatButtonAction
import uniffi.gemstone.GemFiatButtonState
import uniffi.gemstone.GemFiatQuotePhase
import uniffi.gemstone.GemFiatSession

data class FiatUiState(
    val phase: GemFiatQuotePhase = GemFiatQuotePhase.NoInput,
    val amountCheck: GemFiatAmountCheck = GemFiatAmountCheck.Valid,
    val buttonAction: GemFiatButtonAction = GemFiatButtonAction.CONTINUE,
    val buttonState: ButtonState = ButtonState.Disabled,
    val canSelectProvider: Boolean = false,
)

internal fun createFiatUiState(session: GemFiatSession, isUrlLoading: Boolean) = FiatUiState(
    phase = session.current().phase,
    amountCheck = session.amountCheck(),
    buttonAction = session.buttonAction(),
    buttonState = when (session.buttonState(isUrlLoading)) {
        GemFiatButtonState.DISABLED -> ButtonState.Disabled
        GemFiatButtonState.LOADING -> ButtonState.Loading
        GemFiatButtonState.ENABLED -> ButtonState.Enabled
    },
    canSelectProvider = session.canSelectProvider(),
)
