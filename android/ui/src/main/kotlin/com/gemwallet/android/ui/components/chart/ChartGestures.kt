package com.gemwallet.android.ui.components.chart

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.systemGestureExclusion
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.AwaitPointerEventScope
import androidx.compose.ui.input.pointer.PointerEvent
import androidx.compose.ui.input.pointer.PointerInputChange
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalHapticFeedback

private const val FADE_IN_MS = 150
private const val FADE_OUT_MS = 200
private const val SCRUB_HOLD_MS = 100L

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

private enum class ChartTouch {
    Tap,
    Scroll,
    Scrub,
    Pinch,
}

fun Modifier.chartGestures(indexAt: (Float) -> Int?, onSelectionChanged: (Int?) -> Unit, onZoom: (Float) -> Unit): Modifier = this
    .systemGestureExclusion()
    .pointerInput(Unit) {
        awaitEachGesture {
            val down = awaitFirstDown(requireUnconsumed = false)
            when (withTimeoutOrNull(SCRUB_HOLD_MS) { awaitIntent(down, viewConfiguration.touchSlop) } ?: ChartTouch.Scrub) {
                ChartTouch.Scrub -> {
                    indexAt(down.position.x)?.let(onSelectionChanged)
                    val pinched = followScrub(indexAt, onSelectionChanged)
                    onSelectionChanged(null)
                    if (pinched) followPinch(onZoom)
                }

                ChartTouch.Pinch -> followPinch(onZoom)

                ChartTouch.Tap, ChartTouch.Scroll -> Unit
            }
        }
    }

private suspend fun AwaitPointerEventScope.awaitIntent(down: PointerInputChange, touchSlop: Float): ChartTouch {
    while (true) {
        val event = awaitPointerEvent()
        val finger = event.changes.firstOrNull { it.id == down.id }
        when {
            event.pressedCount() > 1 -> return ChartTouch.Pinch
            finger == null || !finger.pressed -> return ChartTouch.Tap
            (finger.position - down.position).getDistance() > touchSlop -> return ChartTouch.Scroll
        }
    }
}

private suspend fun AwaitPointerEventScope.followScrub(indexAt: (Float) -> Int?, onSelectionChanged: (Int?) -> Unit): Boolean {
    while (true) {
        val event = awaitPointerEvent()
        if (event.pressedCount() > 1) return true
        val finger = event.changes.firstOrNull { it.pressed } ?: return false
        finger.consume()
        indexAt(finger.position.x)?.let(onSelectionChanged)
    }
}

private suspend fun AwaitPointerEventScope.followPinch(onZoom: (Float) -> Unit) {
    while (true) {
        val event = awaitPointerEvent()
        if (event.pressedCount() == 0) return
        val zoom = event.calculateZoom()
        if (zoom != 1f) onZoom(zoom)
        event.changes.forEach { it.consume() }
    }
}

private fun PointerEvent.pressedCount(): Int = changes.count { it.pressed }
