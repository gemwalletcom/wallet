package com.gemwallet.android.ui.components.chart

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.remember
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback

private const val FADE_IN_MS = 150
private const val FADE_OUT_MS = 200

@Stable
class ChartSelection internal constructor() {

    private val fade = Animatable(0f)

    val alpha: Float
        get() = fade.value

    internal suspend fun fadeIn() = fade.animateTo(1f, animationSpec = tween(FADE_IN_MS))

    internal suspend fun fadeOut() = fade.animateTo(0f, animationSpec = tween(FADE_OUT_MS))
}

@Composable
fun rememberChartSelection(selectedIndex: Int?): ChartSelection {
    val selection = remember { ChartSelection() }
    val haptic = LocalHapticFeedback.current
    LaunchedEffect(selectedIndex) {
        if (selectedIndex != null) {
            haptic.performHapticFeedback(HapticFeedbackType.SegmentFrequentTick)
            selection.fadeIn()
        } else {
            selection.fadeOut()
        }
    }
    return selection
}
