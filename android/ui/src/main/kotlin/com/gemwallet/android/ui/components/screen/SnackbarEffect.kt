package com.gemwallet.android.ui.components.screen

import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember

@Composable
fun rememberSnackbarState(
    message: String? = null,
    onShown: () -> Unit = {},
): SnackbarHostState {
    val snackbarHostState = remember { SnackbarHostState() }
    LaunchedEffect(message) {
        if (!message.isNullOrEmpty()) {
            snackbarHostState.showSnackbar(message)
            onShown()
        }
    }
    return snackbarHostState
}
