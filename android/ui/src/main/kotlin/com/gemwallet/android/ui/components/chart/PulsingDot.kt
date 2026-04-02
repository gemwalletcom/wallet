package com.gemwallet.android.ui.components.chart

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

private object PulseMetrics {
    const val DURATION_MS = 1800
    const val RING2_PHASE_OFFSET = 0.4f / 1.8f
    const val RING1_MAX_SCALE = 3.0f
    const val RING2_MAX_SCALE = 2.5f
    const val CONTAINER_SCALE = 5
    val ringStroke = 1.5.dp
}

private object PulseAlpha {
    const val RING_STROKE = 0.40f
    const val RING_OPACITY = 0.80f
}

@Composable
fun PulsingDot(
    color: Color,
    modifier: Modifier = Modifier,
    dotSize: Dp = 8.dp,
) {
    val containerSize = dotSize * PulseMetrics.CONTAINER_SCALE
    val transition = rememberInfiniteTransition(label = "pulse")

    val progress by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(PulseMetrics.DURATION_MS, easing = LinearEasing),
            repeatMode = RepeatMode.Restart,
        ),
        label = "ring",
    )

    Box(modifier = modifier.size(containerSize), contentAlignment = Alignment.Center) {
        Canvas(modifier = Modifier.size(containerSize)) {
            val center = Offset(size.width / 2, size.height / 2)
            val dotRadius = dotSize.toPx() / 2
            val ringStrokePx = PulseMetrics.ringStroke.toPx()

            drawPulseRing(center, dotRadius, ringStrokePx, color, progress, PulseMetrics.RING1_MAX_SCALE)
            drawPulseRing(center, dotRadius, ringStrokePx, color, (progress + PulseMetrics.RING2_PHASE_OFFSET) % 1f, PulseMetrics.RING2_MAX_SCALE)

            drawCircle(
                brush = Brush.radialGradient(
                    colors = listOf(Color.White, color),
                    center = center,
                    radius = dotRadius,
                ),
                radius = dotRadius,
                center = center,
            )
        }
    }
}

private fun DrawScope.drawPulseRing(
    center: Offset,
    dotRadius: Float,
    strokeWidth: Float,
    color: Color,
    progress: Float,
    maxScale: Float,
) {
    val eased = easeOutQuad(progress)
    val scale = 1f + (maxScale - 1f) * eased
    val alpha = PulseAlpha.RING_STROKE * PulseAlpha.RING_OPACITY * (1f - eased)
    drawCircle(
        color = color.copy(alpha = alpha),
        radius = dotRadius * scale,
        center = center,
        style = Stroke(width = strokeWidth * scale),
    )
}

private fun easeOutQuad(t: Float): Float = 1f - (1f - t) * (1f - t)
