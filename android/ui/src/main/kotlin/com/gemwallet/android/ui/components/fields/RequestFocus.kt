package com.gemwallet.android.ui.components.fields

import androidx.compose.ui.focus.FocusRequester

fun FocusRequester.requestFocusIfAttached() {
    try {
        requestFocus()
    } catch (_: IllegalStateException) {
    }
}
