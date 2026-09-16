package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.imeAnimationTarget
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalDensity

@OptIn(ExperimentalLayoutApi::class)
val WindowInsets.Companion.keyboard: WindowInsets
    @Composable get() {
        val density = LocalDensity.current
        val height = ime.getBottom(density)
        val isAnimating = height != imeAnimationTarget.getBottom(density)
        var wasOurs by remember { mutableStateOf(height > 0) }
        val isOurs = isAnimating || (wasOurs && height > 0)
        SideEffect { wasOurs = isOurs }
        return if (isOurs) ime else WindowInsets()
    }

val WindowInsets.Companion.isKeyboardVisible: Boolean
    @Composable get() = keyboard.getBottom(LocalDensity.current) > 0
