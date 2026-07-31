package com.gemwallet.android.ui.components.chart

import android.os.Build
import android.view.HapticFeedbackConstants
import android.view.View
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalView

private const val FADE_IN_MS = 150
private const val FADE_OUT_MS = 200

@Stable
class ChartSelection internal constructor(private val view: View) {

    private val fade = Animatable(0f)
    private var lastHapticIndex by mutableIntStateOf(-1)

    val alpha: Float
        get() = fade.value

    internal suspend fun fadeIn() = fade.animateTo(1f, animationSpec = tween(FADE_IN_MS))

    internal suspend fun fadeOut() {
        fade.animateTo(0f, animationSpec = tween(FADE_OUT_MS))
        lastHapticIndex = -1
    }

    internal fun hapticOnStart(index: Int) {
        hapticTick()
        lastHapticIndex = index
    }

    internal fun hapticOnChange(index: Int) {
        if (index != lastHapticIndex) {
            hapticTick()
            lastHapticIndex = index
        }
    }

    private fun hapticTick() {
        view.performHapticFeedback(
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) HapticFeedbackConstants.CLOCK_TICK
            else HapticFeedbackConstants.VIRTUAL_KEY
        )
    }
}

@Composable
fun rememberChartSelection(selectedIndex: Int?): ChartSelection {
    val view = LocalView.current
    val selection = remember(view) { ChartSelection(view) }
    LaunchedEffect(selectedIndex) {
        if (selectedIndex != null) selection.fadeIn() else selection.fadeOut()
    }
    return selection
}

fun Modifier.chartSelection(
    selection: ChartSelection,
    vararg keys: Any?,
    indexAt: (Float) -> Int?,
    onSelectionChanged: (Int?) -> Unit,
): Modifier = this
    .pointerInput(*keys) {
        detectTapGestures(onPress = { touch ->
            indexAt(touch.x)?.let { index ->
                selection.hapticOnChange(index)
                onSelectionChanged(index)
            }
            tryAwaitRelease()
            onSelectionChanged(null)
        })
    }
    .pointerInput(*keys) {
        detectDragGestures(
            onDragStart = { touch ->
                indexAt(touch.x)?.let { index ->
                    selection.hapticOnStart(index)
                    onSelectionChanged(index)
                }
            },
            onDrag = { change, _ ->
                change.consume()
                indexAt(change.position.x)?.let { index ->
                    selection.hapticOnChange(index)
                    onSelectionChanged(index)
                }
            },
            onDragEnd = { onSelectionChanged(null) },
            onDragCancel = { onSelectionChanged(null) },
        )
    }
