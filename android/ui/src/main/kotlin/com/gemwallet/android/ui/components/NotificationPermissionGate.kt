package com.gemwallet.android.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue

@Composable
fun rememberNotificationPermissionGate(onGranted: () -> Unit = {}): (() -> Unit) -> Unit {
    var requesting by remember { mutableStateOf(false) }

    if (requesting) {
        PushRequest(
            onNotificationEnable = {
                requesting = false
                onGranted()
            },
            onDismiss = { requesting = false },
        )
    }

    return { action ->
        action()
        requesting = true
    }
}
