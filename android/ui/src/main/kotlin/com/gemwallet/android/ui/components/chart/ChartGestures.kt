package com.gemwallet.android.ui.components.chart

import androidx.compose.animation.core.AnimationState
import androidx.compose.animation.core.animateDecay
import androidx.compose.animation.core.exponentialDecay
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.systemGestureExclusion
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.AwaitPointerEventScope
import androidx.compose.ui.input.pointer.PointerEvent
import androidx.compose.ui.input.pointer.PointerInputChange
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.util.VelocityTracker
import kotlinx.coroutines.Job
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlin.math.abs

private const val SCRUB_HOLD_MS = 250L
private const val GLIDE_FRICTION = 2.4f

private enum class ChartTouch {
    Tap,
    Scroll,
    Pan,
    Scrub,
    Pinch,
}

fun Modifier.chartGestures(plotLeft: Float, plotWidth: Float, indexAt: (Float) -> Int?, onSelectionChanged: (Int?) -> Unit, onZoom: (Float, Float) -> Unit, onPan: (Float) -> Unit): Modifier = this
    .systemGestureExclusion()
    .pointerInput(plotLeft, plotWidth) {
        val indexAtPixel = { x: Float -> indexAt((x - plotLeft) / plotWidth) }
        val zoomAtPixel = { magnification: Float, x: Float -> onZoom(magnification, (x - plotLeft) / plotWidth) }
        val panByPixels = { distance: Float -> onPan(distance / plotWidth) }
        coroutineScope {
            var glide: Job? = null
            awaitEachGesture {
                val down = awaitFirstDown(requireUnconsumed = false)
                glide?.cancel()
                when (withTimeoutOrNull(SCRUB_HOLD_MS) { awaitIntent(down, viewConfiguration.touchSlop) } ?: ChartTouch.Scrub) {
                    ChartTouch.Scrub -> {
                        indexAtPixel(down.position.x)?.let(onSelectionChanged)
                        val pinched = followScrub(indexAtPixel, onSelectionChanged)
                        onSelectionChanged(null)
                        if (pinched) followPinch(zoomAtPixel)
                    }

                    ChartTouch.Pinch -> followPinch(zoomAtPixel)

                    ChartTouch.Pan -> {
                        val velocity = followPan(down, zoomAtPixel, panByPixels)
                        glide = launch { glide(velocity, panByPixels) }
                    }

                    ChartTouch.Tap, ChartTouch.Scroll -> Unit
                }
            }
        }
    }

private suspend fun AwaitPointerEventScope.awaitIntent(down: PointerInputChange, touchSlop: Float): ChartTouch {
    while (true) {
        val event = awaitPointerEvent()
        val finger = event.changes.firstOrNull { it.id == down.id }
        val moved = finger?.let { it.position - down.position }
        when {
            event.pressedCount() > 1 -> return ChartTouch.Pinch
            finger == null || !finger.pressed || moved == null -> return ChartTouch.Tap
            moved.getDistance() > touchSlop -> return if (abs(moved.x) > abs(moved.y)) ChartTouch.Pan else ChartTouch.Scroll
        }
    }
}

private suspend fun AwaitPointerEventScope.followPan(down: PointerInputChange, onZoom: (Float, Float) -> Unit, onPan: (Float) -> Unit): Float {
    val tracker = VelocityTracker()
    tracker.addPosition(down.uptimeMillis, down.position)
    while (true) {
        val event = awaitPointerEvent()
        if (event.pressedCount() > 1) {
            followPinch(onZoom)
            return 0f
        }
        val finger = event.changes.firstOrNull { it.id == down.id }
        if (finger == null || !finger.pressed) return tracker.calculateVelocity().x
        tracker.addPosition(finger.uptimeMillis, finger.position)
        onPan(finger.position.x - finger.previousPosition.x)
        finger.consume()
    }
}

private suspend fun glide(velocity: Float, onPan: (Float) -> Unit) {
    var travelled = 0f
    AnimationState(initialValue = 0f, initialVelocity = velocity).animateDecay(exponentialDecay(frictionMultiplier = GLIDE_FRICTION)) {
        onPan(value - travelled)
        travelled = value
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

private suspend fun AwaitPointerEventScope.followPinch(onZoom: (Float, Float) -> Unit) {
    while (true) {
        val event = awaitPointerEvent()
        if (event.pressedCount() == 0) return
        val zoom = event.calculateZoom()
        if (zoom != 1f) onZoom(zoom, event.calculateCentroid().x)
        event.changes.forEach { it.consume() }
    }
}

private fun PointerEvent.pressedCount(): Int = changes.count { it.pressed }
