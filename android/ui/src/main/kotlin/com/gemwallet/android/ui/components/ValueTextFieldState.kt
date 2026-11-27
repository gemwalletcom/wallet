package com.gemwallet.android.ui.components

import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.TextRange

@Composable
internal fun rememberValueTextFieldState(value: String): TextFieldState {
    val state = rememberTextFieldState(value, TextRange(value.length))
    if (state.text.toString() != value) {
        state.setTextAndPlaceCursorAtEnd(value)
    }
    return state
}

internal fun valueChangeTransformation(value: String, onValueChange: (String) -> Unit) = InputTransformation {
    val text = asCharSequence().toString()
    if (text != value) onValueChange(text)
}
