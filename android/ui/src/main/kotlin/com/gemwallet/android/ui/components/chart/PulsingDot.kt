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
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

private object PulseMetrics {
    const val CONTAINER_SCALE = 5
    const val DURATION_MS = 1800
    const val SECOND_RING_DELAY_MS = 400
    val ringStroke = 1.5.dp
    val innerGlowRadius = 6.dp   // iOS: .shadow(radius: 6)
    val outerGlowRadius = 12.dp  // iOS: .shadow(radius: 12)
    const val RING1_MAX_SCALE = 3f
    const val RING2_MAX_SCALE = 2.5f
}

private object PulseAlpha {
    const val INNER_GLOW = 0.80f  // iOS: .opacity(.strong)
    const val OUTER_GLOW = 0.40f  // iOS: .opacity(.semiMedium)
    const val RING_BASE = 0.32f   // 0.40 * 0.80
}

@Composable
fun PulsingDot(
    color: Color,
    modifier: Modifier = Modifier,
    dotSize: Dp = 8.dp,
) {
    val containerSize = dotSize * PulseMetrics.CONTAINER_SCALE
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")

    val progress1 by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(PulseMetrics.DURATION_MS, easing = LinearEasing),
            repeatMode = RepeatMode.Restart,
        ),
        label = "ring1",
    )
    val progress2 by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(PulseMetrics.DURATION_MS, delayMillis = PulseMetrics.SECOND_RING_DELAY_MS, easing = LinearEasing),
            repeatMode = RepeatMode.Restart,
        ),
        label = "ring2",
    )

    Box(modifier = modifier.size(containerSize), contentAlignment = Alignment.Center) {
        Canvas(modifier = Modifier.size(containerSize)) {
            val center = Offset(size.width / 2, size.height / 2)
            val dotRadius = dotSize.toPx() / 2
            val ringStrokePx = PulseMetrics.ringStroke.toPx()

            val outerGlowR = dotRadius + PulseMetrics.outerGlowRadius.toPx()
            drawCircle(
                brush = Brush.radialGradient(
                    colorStops = arrayOf(0f to color.copy(alpha = PulseAlpha.OUTER_GLOW), 1f to color.copy(alpha = 0f)),
                    center = center,
                    radius = outerGlowR,
                ),
                radius = outerGlowR,
                center = center,
            )
            val innerGlowR = dotRadius + PulseMetrics.innerGlowRadius.toPx()
            drawCircle(
                brush = Brush.radialGradient(
                    colorStops = arrayOf(0f to color.copy(alpha = PulseAlpha.INNER_GLOW), 1f to color.copy(alpha = 0f)),
                    center = center,
                    radius = innerGlowR,
                ),
                radius = innerGlowR,
                center = center,
            )

            drawPulseRing(center, dotRadius, ringStrokePx, color, progress1, PulseMetrics.RING1_MAX_SCALE)
            drawPulseRing(center, dotRadius, ringStrokePx, color, progress2, PulseMetrics.RING2_MAX_SCALE)

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

private fun androidx.compose.ui.graphics.drawscope.DrawScope.drawPulseRing(
    center: Offset,
    dotRadius: Float,
    strokeWidth: Float,
    color: Color,
    progress: Float,
    maxScale: Float,
) {
    val eased = easeOutQuad(progress)
    val scale = 1f + (maxScale - 1f) * eased
    drawCircle(
        color = color.copy(alpha = PulseAlpha.RING_BASE * (1f - eased)),
        radius = dotRadius * scale,
        center = center,
        style = Stroke(width = strokeWidth),
    )
}

private fun easeOutQuad(t: Float): Float = 1f - (1f - t) * (1f - t)
