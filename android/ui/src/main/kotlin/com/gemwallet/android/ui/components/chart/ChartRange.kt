package com.gemwallet.android.ui.components.chart

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

private const val RANGE_SETTLE_FRACTION = 0.005f
private const val RANGE_GROW_STIFFNESS = 600f

@Immutable
internal data class ChartViewport(val low: Float, val span: Float) {
    fun fraction(value: Double): Float = ((value - low) / span).toFloat()

    fun y(value: Float, height: Float): Float = y(value.toDouble(), height)

    fun y(value: Double, height: Float): Float = height - fraction(value) * height
}

@Stable
class ChartRange internal constructor(low: Float, high: Float) {

    private val lowEdge = Animatable(minOf(low, high))
    private val highEdge = Animatable(maxOf(low, high))

    val low: Float
        get() = minOf(lowEdge.value, highEdge.value)

    val high: Float
        get() = maxOf(lowEdge.value, highEdge.value)

    internal val viewport: ChartViewport?
        get() {
            val low = low
            val span = high - low
            return if (low.isFinite() && span.isFinite() && span > 0f) ChartViewport(low, span) else null
        }

    internal fun follow(scope: CoroutineScope, low: Float, high: Float) {
        val targetLow = minOf(low, high)
        val targetHigh = maxOf(low, high)
        val threshold = (targetHigh - targetLow) * RANGE_SETTLE_FRACTION
        scope.launch { lowEdge.animateTo(targetLow, settle(growing = targetLow < lowEdge.value, threshold)) }
        scope.launch { highEdge.animateTo(targetHigh, settle(growing = targetHigh > highEdge.value, threshold)) }
    }

    private fun settle(growing: Boolean, threshold: Float) = spring(
        dampingRatio = Spring.DampingRatioNoBouncy,
        stiffness = if (growing) RANGE_GROW_STIFFNESS else Spring.StiffnessLow,
        visibilityThreshold = threshold,
    )
}

@Composable
fun rememberChartRange(low: Float, high: Float): ChartRange {
    val range = remember { ChartRange(low, high) }
    val target by rememberUpdatedState(low to high)
    LaunchedEffect(range) {
        snapshotFlow { target }.collect { (low, high) -> range.follow(this, low, high) }
    }
    return range
}
