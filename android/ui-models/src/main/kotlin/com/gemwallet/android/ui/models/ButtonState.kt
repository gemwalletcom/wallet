package com.gemwallet.android.ui.models

import uniffi.gemstone.GemButtonState

enum class ButtonState {
    Enabled,
    Loading,
    Disabled,
}

fun buttonState(enabled: Boolean = true, loading: Boolean = false): ButtonState = when {
    loading -> ButtonState.Loading
    enabled -> ButtonState.Enabled
    else -> ButtonState.Disabled
}

fun GemButtonState.buttonState(): ButtonState = when (this) {
    GemButtonState.DISABLED -> ButtonState.Disabled
    GemButtonState.LOADING -> ButtonState.Loading
    GemButtonState.ENABLED -> ButtonState.Enabled
}
