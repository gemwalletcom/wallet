package com.gemwallet.android.ui.models

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
