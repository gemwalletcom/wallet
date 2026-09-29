package com.gemwallet.android.ui.components.chart

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.runtime.Composable
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

@Stable
class ChartRange internal constructor(low: Float, high: Float) {

    private val lowEdge = Animatable(low)
    private val highEdge = Animatable(high)

    val low: Float
        get() = lowEdge.value

    val high: Float
        get() = highEdge.value

    internal fun follow(scope: CoroutineScope, low: Float, high: Float) {
        val threshold = (high - low) * RANGE_SETTLE_FRACTION
        scope.launch { lowEdge.animateTo(low, settle(growing = low < lowEdge.value, threshold)) }
        scope.launch { highEdge.animateTo(high, settle(growing = high > highEdge.value, threshold)) }
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
