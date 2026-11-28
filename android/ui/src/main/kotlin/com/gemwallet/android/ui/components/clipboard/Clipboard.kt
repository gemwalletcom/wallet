package com.gemwallet.android.ui.components.clipboard

import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Stable
import androidx.compose.runtime.remember
import androidx.compose.ui.hapticfeedback.HapticFeedback
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import uniffi.gemstone.GemCopy

@Stable
class Clipboard internal constructor(private val context: Context, private val haptic: HapticFeedback) {
    private val manager = context.clipboardManager()

    fun copy(copy: GemCopy) {
        manager.setCopy(context, copy)
        haptic.performHapticFeedback(HapticFeedbackType.Confirm)
    }

    fun copy(text: String) {
        manager.setPlainText(context, text)
        haptic.performHapticFeedback(HapticFeedbackType.Confirm)
    }

    fun paste(): String? {
        val text = manager.getPlainText()
        if (text == null) {
            haptic.performHapticFeedback(HapticFeedbackType.Reject)
        }
        return text
    }

    fun clear() = manager.clear()
}

@Composable
fun rememberClipboard(): Clipboard {
    val context = LocalContext.current
    val haptic = LocalHapticFeedback.current
    return remember(context, haptic) { Clipboard(context, haptic) }
}
