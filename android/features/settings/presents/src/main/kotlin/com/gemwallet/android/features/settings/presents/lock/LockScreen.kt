package com.gemwallet.android.features.settings.presents.lock

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import com.gemwallet.android.ui.theme.setSystemBarsAppearance

@Composable
fun LockScreen(darkTheme: Boolean, onLeave: () -> Unit, content: @Composable () -> Unit) {
    Dialog(
        onDismissRequest = onLeave,
        properties = DialogProperties(dismissOnClickOutside = false, usePlatformDefaultWidth = false, decorFitsSystemWindows = false),
    ) {
        val window = (LocalView.current.parent as DialogWindowProvider).window
        SideEffect { window.setSystemBarsAppearance(darkTheme) }
        content()
    }
}

@Composable
fun UnlockedContent(isUnlocked: Boolean, content: @Composable () -> Unit) {
    var hasUnlocked by remember { mutableStateOf(isUnlocked) }
    LaunchedEffect(isUnlocked) {
        if (isUnlocked) hasUnlocked = true
    }
    if (hasUnlocked) content()
}
