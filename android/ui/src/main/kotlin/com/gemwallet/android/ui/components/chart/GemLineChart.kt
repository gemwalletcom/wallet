package com.gemwallet.android.ui.components.chart

import android.os.Build
import android.view.HapticFeedbackConstants
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Fill
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.abs
import kotlin.math.min
import kotlin.math.roundToInt
import kotlin.math.sqrt

data class ChartPoint(
    val x: Float,
    val y: Float,
)

private object Metrics {
    val lineWidth = 2.5.dp
    val selectionDotRadius = 6.dp
    val dashLength = 4.dp
    val labelWidth = 88.dp
    val verticalPadding = 24.dp
    val horizontalPadding = 16.dp
    val boundLabelOffsetBelow = 6.dp
    val boundLabelOffsetAbove = 18.dp
    val selectionGlowExtra = 6.dp
    val pulsingDotSize = 8.dp
    const val MAX_RENDER_POINTS = 120
    const val X_RIGHT_PADDING_FRACTION = 0.02f
    const val FLAT_LINE_PADDING = 0.01f
    const val FLAT_LINE_MIN_RANGE = 0.01f
    const val RANGE_PADDING = 0.05f
    const val FADE_IN_MS = 150
    const val FADE_OUT_MS = 200
}

private object Alpha {
    const val SELECTION_LINE = 0.50f
    const val GLOW_CENTER = 0.40f
    const val GLOW_INNER = 0.15f
    const val GLOW_OUTER = 0.04f
    const val DOT_BORDER = 0.80f
    val GRADIENT_LIGHT = floatArrayOf(0.45f, 0.38f, 0.28f, 0.15f, 0.05f)
    val GRADIENT_DARK = floatArrayOf(0.30f, 0.22f, 0.14f, 0.07f, 0.02f)
}

@Composable
fun GemLineChart(
    points: List<ChartPoint>,
    lineColor: Color,
    modifier: Modifier = Modifier,
    selectedIndex: Int? = null,
    onSelectionChanged: (Int?) -> Unit = {},
    minLabel: String? = null,
    maxLabel: String? = null,
) {
    if (points.size < 2) return

    val density = LocalDensity.current
    val view = LocalView.current
    val textMeasurer = rememberTextMeasurer()
    val isDark = isSystemInDarkTheme()

    val lineWidthPx = with(density) { Metrics.lineWidth.toPx() }
    val horizontalPaddingPx = with(density) { Metrics.horizontalPadding.toPx() }
    val selectionDotRadiusPx = with(density) { Metrics.selectionDotRadius.toPx() }
    val dashLengthPx = with(density) { Metrics.dashLength.toPx() }
    val labelWidthPx = with(density) { Metrics.labelWidth.toPx() }
    val verticalPaddingPx = with(density) { Metrics.verticalPadding.toPx() }
    val labelOffsetBelowPx = with(density) { Metrics.boundLabelOffsetBelow.toPx() }
    val labelOffsetAbovePx = with(density) { Metrics.boundLabelOffsetAbove.toPx() }
    val glowExtraPx = with(density) { Metrics.selectionGlowExtra.toPx() }

    val yMin = points.minOf { it.y }
    val yMax = points.maxOf { it.y }
    val (paddedMin, paddedRange) = calculatePaddedRange(yMin, yMax)

    val minIndex = points.indexOfFirst { it.y == yMin }
    val maxIndex = points.indexOfFirst { it.y == yMax }

    val renderPoints = remember(points) {
        if (points.size > Metrics.MAX_RENDER_POINTS) reducePoints(points, Metrics.MAX_RENDER_POINTS) else points
    }

    var chartSize by remember { mutableStateOf(IntSize.Zero) }
    var lastHapticIndex by remember { mutableIntStateOf(-1) }

    val selectionAlpha = remember { Animatable(0f) }
    LaunchedEffect(selectedIndex) {
        if (selectedIndex != null) {
            selectionAlpha.animateTo(1f, animationSpec = tween(Metrics.FADE_IN_MS))
        } else {
            selectionAlpha.animateTo(0f, animationSpec = tween(Metrics.FADE_OUT_MS))
            lastHapticIndex = -1
        }
    }

    val labelStyle = TextStyle(
        color = MaterialTheme.colorScheme.secondary,
        fontSize = 11.sp,
        textAlign = TextAlign.Center,
    )

    Box(modifier = modifier.fillMaxSize().onSizeChanged { chartSize = it }) {
        if (chartSize.width > 0 && chartSize.height > 0) {
            val canvasWidth = chartSize.width.toFloat()
            val canvasHeight = chartSize.height.toFloat()
            val plotTop = verticalPaddingPx
            val plotBottom = canvasHeight - verticalPaddingPx
            val plotHeight = plotBottom - plotTop
            if (plotHeight <= 0) return@Box

            val curveLeft = horizontalPaddingPx
            val curveWidth = (canvasWidth - 2 * horizontalPaddingPx) * (1f - Metrics.X_RIGHT_PADDING_FRACTION)
            val lastPointX = if (points.size > 1) (points.size - 1).toFloat() else 1f

            fun curveScreenX(index: Int, count: Int) = curveLeft + indexToX(index, count, curveWidth)
            fun renderScreenX(originalX: Float) = curveLeft + originalX / lastPointX * curveWidth
            fun valueToScreenY(value: Float) = plotTop + valueToY(value, paddedMin, paddedRange, plotHeight)

            Canvas(
                modifier = Modifier
                    .fillMaxSize()
                    .pointerInput(points) {
                        detectTapGestures(onPress = { touch ->
                            findClosestIndex(points, touch.x, curveLeft, curveWidth)?.let { index ->
                                if (index != lastHapticIndex) { haptic(view); lastHapticIndex = index }
                                onSelectionChanged(index)
                            }
                            tryAwaitRelease()
                            onSelectionChanged(null)
                        })
                    }
                    .pointerInput(points) {
                        detectDragGestures(
                            onDragStart = { touch ->
                                findClosestIndex(points, touch.x, curveLeft, curveWidth)?.let { index ->
                                    haptic(view); lastHapticIndex = index; onSelectionChanged(index)
                                }
                            },
                            onDrag = { change, _ ->
                                change.consume()
                                findClosestIndex(points, change.position.x, curveLeft, curveWidth)?.let { index ->
                                    if (index != lastHapticIndex) { haptic(view); lastHapticIndex = index }
                                    onSelectionChanged(index)
                                }
                            },
                            onDragEnd = { onSelectionChanged(null) },
                            onDragCancel = { onSelectionChanged(null) },
                        )
                    }
            ) {
                val screenPoints = renderPoints.map { point ->
                    Offset(renderScreenX(point.x), valueToScreenY(point.y))
                }
                val curvePath = buildCurvePath(screenPoints)

                val screenYRange = screenPoints.maxOf { it.y } - screenPoints.minOf { it.y }
                if (screenYRange > 2f) {
                    drawAreaGradient(curvePath, screenPoints, plotBottom, plotTop, lineColor, isDark)
                }
                drawPath(curvePath, lineColor, style = Stroke(lineWidthPx, cap = StrokeCap.Round, join = StrokeJoin.Round))

                if (minLabel != null && minIndex in points.indices) {
                    drawBoundLabel(textMeasurer, minLabel, labelStyle, curveScreenX(minIndex, points.size), plotBottom + labelOffsetBelowPx, canvasWidth, labelWidthPx)
                }
                if (maxLabel != null && maxIndex in points.indices) {
                    drawBoundLabel(textMeasurer, maxLabel, labelStyle, curveScreenX(maxIndex, points.size), plotTop - labelOffsetAbovePx, canvasWidth, labelWidthPx)
                }

                if (selectedIndex != null && selectedIndex in points.indices) {
                    drawSelectionIndicator(
                        Offset(curveScreenX(selectedIndex, points.size), valueToScreenY(points[selectedIndex].y)),
                        canvasHeight, selectionAlpha.value, lineColor, lineWidthPx, selectionDotRadiusPx, dashLengthPx, glowExtraPx,
                    )
                }
            }

            if (selectedIndex == null && points.isNotEmpty()) {
                val dotContainerPx = with(density) { (Metrics.pulsingDotSize * 5).toPx() }
                val lastX = curveScreenX(points.lastIndex, points.size)
                val lastY = valueToScreenY(points.last().y)
                PulsingDot(
                    color = lineColor,
                    dotSize = Metrics.pulsingDotSize,
                    modifier = Modifier.offset {
                        IntOffset((lastX - dotContainerPx / 2).roundToInt(), (lastY - dotContainerPx / 2).roundToInt())
                    },
                )
            }
        }
    }
}

private fun DrawScope.drawSelectionIndicator(
    point: Offset,
    canvasHeight: Float,
    alpha: Float,
    color: Color,
    lineWidth: Float,
    dotRadius: Float,
    dashLength: Float,
    glowExtra: Float,
) {
    drawLine(
        color.copy(Alpha.SELECTION_LINE * alpha),
        Offset(point.x, 0f),
        Offset(point.x, canvasHeight),
        1.dp.toPx(),
        pathEffect = PathEffect.dashPathEffect(floatArrayOf(dashLength, dashLength)),
    )
    val glowRadius = dotRadius + glowExtra
    drawCircle(
        Brush.radialGradient(
            colorStops = arrayOf(
                0f to color.copy(Alpha.GLOW_CENTER * alpha),
                0.4f to color.copy(Alpha.GLOW_INNER * alpha),
                0.7f to color.copy(Alpha.GLOW_OUTER * alpha),
                1f to color.copy(0f),
            ),
            center = point,
            radius = glowRadius,
        ),
        glowRadius,
        point,
    )
    drawCircle(
        Brush.radialGradient(listOf(Color.White.copy(alpha), color.copy(Alpha.DOT_BORDER * alpha)), point, dotRadius),
        dotRadius,
        point,
    )
    drawCircle(color.copy(alpha), dotRadius, point, style = Stroke(lineWidth))
}

private fun DrawScope.drawAreaGradient(
    curvePath: Path,
    screenPoints: List<Offset>,
    bottomY: Float,
    topY: Float,
    color: Color,
    isDark: Boolean,
) {
    if (screenPoints.size < 2) return
    val areaPath = Path().apply {
        addPath(curvePath)
        lineTo(screenPoints.last().x, bottomY)
        lineTo(screenPoints.first().x, bottomY)
        close()
    }
    val gradientAlphas = if (isDark) Alpha.GRADIENT_DARK else Alpha.GRADIENT_LIGHT
    drawPath(
        areaPath,
        Brush.verticalGradient(
            colorStops = arrayOf(
                0.00f to color.copy(gradientAlphas[0]),
                0.25f to color.copy(gradientAlphas[1]),
                0.50f to color.copy(gradientAlphas[2]),
                0.75f to color.copy(gradientAlphas[3]),
                0.92f to color.copy(gradientAlphas[4]),
                1.00f to color.copy(0f),
            ),
            startY = topY,
            endY = bottomY,
        ),
        style = Fill,
    )
}

private fun DrawScope.drawBoundLabel(
    measurer: TextMeasurer,
    text: String,
    style: TextStyle,
    anchorX: Float,
    anchorY: Float,
    canvasWidth: Float,
    labelWidth: Float,
) {
    val measured = measurer.measure(text, style)
    val halfLabel = labelWidth / 2
    val labelX = (if (anchorX < halfLabel) anchorX - halfLabel / 2 else min(anchorX - halfLabel, canvasWidth - labelWidth)).coerceAtLeast(0f)
    val textX = labelX + (labelWidth - measured.size.width) / 2
    val clampedY = anchorY.coerceIn(0f, size.height - measured.size.height.toFloat())
    drawText(measured, topLeft = Offset(textX, clampedY))
}

private fun buildCurvePath(screenPoints: List<Offset>): Path {
    val path = Path()
    if (screenPoints.size < 2) return path
    path.moveTo(screenPoints[0].x, screenPoints[0].y)
    if (screenPoints.size == 2) {
        path.lineTo(screenPoints[1].x, screenPoints[1].y)
        return path
    }
    for (segment in 0 until screenPoints.size - 1) {
        val prev = screenPoints[(segment - 1).coerceAtLeast(0)]
        val current = screenPoints[segment]
        val next = screenPoints[min(segment + 1, screenPoints.lastIndex)]
        val afterNext = screenPoints[min(segment + 2, screenPoints.lastIndex)]

        val distPrevCurrent = distanceBetween(prev, current).coerceAtLeast(1e-4f)
        val distCurrentNext = distanceBetween(current, next).coerceAtLeast(1e-4f)
        val distNextAfter = distanceBetween(next, afterNext).coerceAtLeast(1e-4f)

        val knot1 = sqrt(distPrevCurrent)
        val knot2 = knot1 + sqrt(distCurrentNext)
        val knot3 = knot2 + sqrt(distNextAfter)

        val tangent1X = (knot2 - knot1) * ((current.x - prev.x) / knot1 - (next.x - prev.x) / knot2 + (next.x - current.x) / (knot2 - knot1))
        val tangent1Y = (knot2 - knot1) * ((current.y - prev.y) / knot1 - (next.y - prev.y) / knot2 + (next.y - current.y) / (knot2 - knot1))
        val tangent2X = (knot2 - knot1) * ((next.x - current.x) / (knot2 - knot1) - (afterNext.x - current.x) / (knot3 - knot1) + (afterNext.x - next.x) / (knot3 - knot2))
        val tangent2Y = (knot2 - knot1) * ((next.y - current.y) / (knot2 - knot1) - (afterNext.y - current.y) / (knot3 - knot1) + (afterNext.y - next.y) / (knot3 - knot2))

        path.cubicTo(
            current.x + tangent1X / 3f, current.y + tangent1Y / 3f,
            next.x - tangent2X / 3f, next.y - tangent2Y / 3f,
            next.x, next.y,
        )
    }
    return path
}

private fun distanceBetween(from: Offset, to: Offset): Float =
    sqrt((from.x - to.x).let { it * it } + (from.y - to.y).let { it * it })

private fun reducePoints(data: List<ChartPoint>, targetCount: Int): List<ChartPoint> {
    if (data.size <= targetCount) return data
    val result = ArrayList<ChartPoint>(targetCount)
    result.add(data.first())
    val bucketSize = (data.size - 2).toDouble() / (targetCount - 2)
    var previousSelectedIndex = 0
    for (bucketIndex in 1 until targetCount - 1) {
        val bucketStart = ((bucketIndex - 1) * bucketSize + 1).toInt()
        val bucketEnd = (bucketIndex * bucketSize + 1).toInt().coerceAtMost(data.size - 1)
        val nextBucketStart = (bucketIndex * bucketSize + 1).toInt()
        val nextBucketEnd = ((bucketIndex + 1) * bucketSize + 1).toInt().coerceAtMost(data.size - 1)
        var avgX = 0f; var avgY = 0f; var count = 0
        for (index in nextBucketStart..nextBucketEnd) { avgX += data[index].x; avgY += data[index].y; count++ }
        if (count > 0) { avgX /= count; avgY /= count }
        val previousPoint = data[previousSelectedIndex]
        var largestArea = -1f; var largestAreaIndex = bucketStart
        for (index in bucketStart..bucketEnd) {
            val area = abs((previousPoint.x - avgX) * (data[index].y - previousPoint.y) - (previousPoint.x - data[index].x) * (avgY - previousPoint.y))
            if (area > largestArea) { largestArea = area; largestAreaIndex = index }
        }
        result.add(data[largestAreaIndex]); previousSelectedIndex = largestAreaIndex
    }
    result.add(data.last())
    return result
}

private fun calculatePaddedRange(min: Float, max: Float): Pair<Float, Float> {
    val range = max - min
    return if (range == 0f) {
        val padding = (min * Metrics.FLAT_LINE_PADDING).coerceAtLeast(Metrics.FLAT_LINE_MIN_RANGE)
        (min - padding) to (padding * 2f)
    } else {
        val padding = range * Metrics.RANGE_PADDING
        (min - padding) to (range + padding * 2f)
    }
}

private fun indexToX(index: Int, count: Int, width: Float): Float =
    if (count <= 1) width / 2 else index.toFloat() / (count - 1) * width

private fun valueToY(value: Float, minValue: Float, range: Float, height: Float): Float =
    height - ((value - minValue) / range) * height

private fun findClosestIndex(points: List<ChartPoint>, touchX: Float, curveLeft: Float, curveWidth: Float): Int? =
    points.indices.minByOrNull { index ->
        abs(curveLeft + indexToX(index, points.size, curveWidth) - touchX)
    }

private fun haptic(view: android.view.View) {
    view.performHapticFeedback(
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) HapticFeedbackConstants.CLOCK_TICK
        else HapticFeedbackConstants.VIRTUAL_KEY
    )
}
