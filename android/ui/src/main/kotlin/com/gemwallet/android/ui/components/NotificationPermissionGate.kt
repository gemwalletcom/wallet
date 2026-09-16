package com.gemwallet.android.ui.components

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue

@Composable
fun rememberNotificationPermissionGate(): (() -> Unit) -> Unit {
    var pending by remember { mutableStateOf<(() -> Unit)?>(null) }

    pending?.let { action ->
        PushRequest(
            onNotificationEnable = {
                pending = null
                action()
            },
            onDismiss = { pending = null },
        )
    }

    return { action -> pending = action }
}
