package com.gemwallet.android.ui.components.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback

@Composable
fun WarningHaptic() {
    val haptic = LocalHapticFeedback.current
    LaunchedEffect(haptic) {
        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
    }
}
