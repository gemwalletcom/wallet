package com.gemwallet.android.ui.components.chart

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.style.color
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space1
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8
import uniffi.gemstone.ChartCandleStick
import uniffi.gemstone.GemCandleChart
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemPerpetualChartLine
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemValueTone
import kotlin.math.max
import kotlin.math.min

private object CandlestickMetrics {
    val topPadding = paddingDefault
    val bottomPadding = space8
    val rightAxisWidth = 88.dp
    val leftPadding = space8
    val labelPadding = space4
    val candleSpacingFraction = 0.25f
    val candleBodyWidth = space4
    val wickWidth = space1
    val referenceLineThickness = space1
    val referenceLineDash = space4
    val referenceLineGap = 3.dp
    val selectionLineWidth = space1
    val selectionDashLength = space4
    val selectionDotOuterRadius = space6
    val selectionDotBorderWidth = space2
    val currentPriceBadgeHorizontalPadding = space2
    val currentPriceBadgeVerticalPadding = space1
    val referenceBadgeHorizontalPadding = space4
    val referenceBadgeVerticalPadding = space4
    val badgeCornerRadius = space4
    val referenceLabelHorizontalGap = space4
    val axisLabelSize = 11.sp

    const val SELECTION_LINE_ALPHA = 0.50f
}

@Composable
fun GemCandlestickChart(chart: GemCandleChart, selectedIndex: Int? = null, onSelectionChanged: (Int?) -> Unit = {}, modifier: Modifier = Modifier) {
    if (chart.candles.isEmpty()) return

    val layout = chart.layout
    val context = LocalContext.current
    val lineLabels = remember(chart) { layout.lines.map { it.label.string(context) } }
    val density = LocalDensity.current
    val textMeasurer = rememberTextMeasurer()

    val upColor = ListItemTextStyle.Positive.color()
    val downColor = ListItemTextStyle.Negative.color()
    val flatColor = ListItemTextStyle.Secondary.color()
    val axisLabelColor = MaterialTheme.colorScheme.secondary
    val gridGuidelineColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.13f)
    val selectionAccentColor = MaterialTheme.colorScheme.primary
    val referenceColorByRole = referenceColors()

    val gridDashEffect = remember(density) {
        val dashPx = with(density) { space4.toPx() }
        PathEffect.dashPathEffect(floatArrayOf(dashPx, dashPx))
    }
    val axisLabelStyle = remember(axisLabelColor) {
        TextStyle(color = axisLabelColor, fontSize = CandlestickMetrics.axisLabelSize, textAlign = TextAlign.Start)
    }
    val whiteLabelStyle = remember(axisLabelStyle) { axisLabelStyle.copy(color = Color.White) }

    val wickWidthPx = with(density) { CandlestickMetrics.wickWidth.toPx() }
    val maxBodyWidthPx = with(density) { CandlestickMetrics.candleBodyWidth.toPx() }
    val referenceLineThicknessPx = with(density) { CandlestickMetrics.referenceLineThickness.toPx() }
    val referenceLineDashPx = with(density) { CandlestickMetrics.referenceLineDash.toPx() }
    val referenceLineGapPx = with(density) { CandlestickMetrics.referenceLineGap.toPx() }
    val labelPaddingPx = with(density) { CandlestickMetrics.labelPadding.toPx() }
    val selectionLineWidthPx = with(density) { CandlestickMetrics.selectionLineWidth.toPx() }
    val selectionDashLengthPx = with(density) { CandlestickMetrics.selectionDashLength.toPx() }
    val selectionDotOuterRadiusPx = with(density) { CandlestickMetrics.selectionDotOuterRadius.toPx() }
    val selectionDotBorderPx = with(density) { CandlestickMetrics.selectionDotBorderWidth.toPx() }
    val currentPriceBadgeHorizontalPaddingPx = with(density) { CandlestickMetrics.currentPriceBadgeHorizontalPadding.toPx() }
    val currentPriceBadgeVerticalPaddingPx = with(density) { CandlestickMetrics.currentPriceBadgeVerticalPadding.toPx() }
    val referenceBadgeHorizontalPaddingPx = with(density) { CandlestickMetrics.referenceBadgeHorizontalPadding.toPx() }
    val referenceBadgeVerticalPaddingPx = with(density) { CandlestickMetrics.referenceBadgeVerticalPadding.toPx() }
    val badgeCornerRadiusPx = with(density) { CandlestickMetrics.badgeCornerRadius.toPx() }
    val referenceLabelGapPx = with(density) { CandlestickMetrics.referenceLabelHorizontalGap.toPx() }
    val topPaddingPx = with(density) { CandlestickMetrics.topPadding.toPx() }
    val bottomPaddingPx = with(density) { CandlestickMetrics.bottomPadding.toPx() }
    val leftPaddingPx = with(density) { CandlestickMetrics.leftPadding.toPx() }
    val rightAxisWidthPx = with(density) { CandlestickMetrics.rightAxisWidth.toPx() }

    var chartSize by remember { mutableStateOf(IntSize.Zero) }
    val selection = rememberChartSelection(selectedIndex)

    Box(modifier = modifier.fillMaxSize().onSizeChanged { chartSize = it }) {
        if (chartSize.width <= 0 || chartSize.height <= 0) return@Box

        val canvasWidth = chartSize.width.toFloat()
        val canvasHeight = chartSize.height.toFloat()
        val plotLeft = leftPaddingPx
        val plotRight = canvasWidth - rightAxisWidthPx
        val plotTop = topPaddingPx
        val plotBottom = canvasHeight - bottomPaddingPx
        val plotWidth = plotRight - plotLeft
        val plotHeight = plotBottom - plotTop
        if (plotWidth <= 0 || plotHeight <= 0) return@Box

        val slotWidth = plotWidth / chart.candles.size
        val bodyWidth = min(
            maxBodyWidthPx,
            max(1f, slotWidth * (1f - CandlestickMetrics.candleSpacingFraction)),
        )

        fun slotCenter(index: Int): Float = plotLeft + slotWidth * (index + 0.5f)
        fun valueToY(value: Double): Float = (plotBottom - (value - layout.priceLow) / (layout.priceHigh - layout.priceLow) * plotHeight).toFloat()
        fun touchToIndex(x: Float): Int? {
            if (x < plotLeft || x > plotRight) return null
            return ((x - plotLeft) / slotWidth).toInt().coerceIn(0, chart.candles.lastIndex)
        }

        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .chartSelection(
                    chartSize,
                    chart.candles.size,
                    indexAt = ::touchToIndex,
                    onSelectionChanged = onSelectionChanged,
                ),
        ) {
            drawYAxis(layout.ticks, ::valueToY, plotLeft, plotRight, gridGuidelineColor, gridDashEffect, textMeasurer, axisLabelStyle, labelPaddingPx)
            drawXAxisGridlines(layout.xTickCount.toInt(), plotLeft, plotRight, plotTop, plotBottom, gridGuidelineColor, gridDashEffect)
            drawCandles(chart.candles, layout.tones, ::slotCenter, ::valueToY, bodyWidth, wickWidthPx, upColor, downColor, flatColor)
            drawReferenceLines(
                referenceLines = layout.lines,
                labels = lineLabels,
                referenceColorByRole = referenceColorByRole,
                valueToY = ::valueToY,
                plotLeft = plotLeft,
                plotRight = plotRight,
                plotTop = plotTop,
                plotBottom = plotBottom,
                lineThicknessPx = referenceLineThicknessPx,
                lineDashEffect = PathEffect.dashPathEffect(floatArrayOf(referenceLineDashPx, referenceLineGapPx)),
                labelStyle = whiteLabelStyle,
                labelPaddingPx = labelPaddingPx,
                labelHorizontalGapPx = referenceLabelGapPx,
                badgeHorizontalPaddingPx = referenceBadgeHorizontalPaddingPx,
                badgeVerticalPaddingPx = referenceBadgeVerticalPaddingPx,
                badgeCornerRadiusPx = badgeCornerRadiusPx,
                textMeasurer = textMeasurer,
            )
            drawCurrentPriceBadge(
                lastCandle = chart.candles.last(),
                lastTone = layout.tones.lastOrNull() ?: GemValueTone.NEUTRAL,
                priceLabel = layout.currentPrice?.text().orEmpty(),
                valueToY = ::valueToY,
                plotTop = plotTop,
                plotBottom = plotBottom,
                plotRight = plotRight,
                labelPaddingPx = labelPaddingPx,
                badgeHorizontalPaddingPx = currentPriceBadgeHorizontalPaddingPx,
                badgeVerticalPaddingPx = currentPriceBadgeVerticalPaddingPx,
                badgeCornerRadiusPx = badgeCornerRadiusPx,
                labelStyle = whiteLabelStyle,
                upColor = upColor,
                downColor = downColor,
                flatColor = flatColor,
                textMeasurer = textMeasurer,
            )
            drawSelection(
                selectedIndex = selectedIndex,
                selectionAlpha = selection.alpha,
                candles = chart.candles,
                slotCenter = ::slotCenter,
                valueToY = ::valueToY,
                plotTop = plotTop,
                plotBottom = plotBottom,
                accentColor = selectionAccentColor,
                lineWidthPx = selectionLineWidthPx,
                dashLengthPx = selectionDashLengthPx,
                dotOuterRadiusPx = selectionDotOuterRadiusPx,
                dotBorderPx = selectionDotBorderPx,
            )
        }
    }
}

@Composable
private fun referenceColors(): (GemPerpetualChartLineKind) -> Color {
    val colors = GemPerpetualChartLineKind.entries.associateWith { it.color() }
    return { role -> colors.getValue(role) }
}

private fun candleColor(tone: GemValueTone, up: Color, down: Color, flat: Color): Color = when (tone) {
    GemValueTone.POSITIVE -> up

    GemValueTone.NEGATIVE -> down

    GemValueTone.NEUTRAL,
    GemValueTone.PLAIN,
    GemValueTone.WARNING,
    -> flat
}

private fun DrawScope.drawYAxis(
    ticks: List<GemFormattedNumber>,
    valueToY: (Double) -> Float,
    plotLeft: Float,
    plotRight: Float,
    guidelineColor: Color,
    guidelineDash: PathEffect,
    textMeasurer: TextMeasurer,
    style: TextStyle,
    labelPaddingPx: Float,
) {
    ticks.forEach { tick ->
        val y = valueToY(tick.value)
        drawLine(
            color = guidelineColor,
            start = Offset(plotLeft, y),
            end = Offset(plotRight, y),
            strokeWidth = 1f,
            pathEffect = guidelineDash,
        )
        val measured = textMeasurer.measure(tick.text(), style)
        drawText(textLayoutResult = measured, topLeft = Offset(plotRight + labelPaddingPx, y - measured.size.height / 2f))
    }
}

private fun DrawScope.drawXAxisGridlines(tickCount: Int, plotLeft: Float, plotRight: Float, plotTop: Float, plotBottom: Float, color: Color, dash: PathEffect) {
    if (tickCount < 2) return
    val plotWidth = plotRight - plotLeft
    (0 until tickCount).forEach { tick ->
        val x = plotLeft + plotWidth * tick / (tickCount - 1)
        drawLine(
            color = color,
            start = Offset(x, plotTop),
            end = Offset(x, plotBottom),
            strokeWidth = 1f,
            pathEffect = dash,
        )
    }
}

private fun DrawScope.drawCandles(
    candles: List<ChartCandleStick>,
    tones: List<GemValueTone>,
    slotCenter: (Int) -> Float,
    valueToY: (Double) -> Float,
    bodyWidth: Float,
    wickWidthPx: Float,
    upColor: Color,
    downColor: Color,
    flatColor: Color,
) {
    candles.zip(tones).forEachIndexed { index, (candle, tone) ->
        val color = candleColor(tone, upColor, downColor, flatColor)
        val centerX = slotCenter(index)
        drawLine(
            color = color,
            start = Offset(centerX, valueToY(candle.high)),
            end = Offset(centerX, valueToY(candle.low)),
            strokeWidth = wickWidthPx,
        )
        val openY = valueToY(candle.open)
        val closeY = valueToY(candle.close)
        val bodyTop = min(openY, closeY)
        val bodyBottom = max(openY, closeY)
        val bodyHeight = max(1f, bodyBottom - bodyTop)
        drawRect(
            color = color,
            topLeft = Offset(centerX - bodyWidth / 2f, bodyTop),
            size = Size(bodyWidth, bodyHeight),
        )
    }
}

private fun DrawScope.drawReferenceLines(
    referenceLines: List<GemPerpetualChartLine>,
    labels: List<String>,
    referenceColorByRole: (GemPerpetualChartLineKind) -> Color,
    valueToY: (Double) -> Float,
    plotLeft: Float,
    plotRight: Float,
    plotTop: Float,
    plotBottom: Float,
    lineThicknessPx: Float,
    lineDashEffect: PathEffect,
    labelStyle: TextStyle,
    labelPaddingPx: Float,
    labelHorizontalGapPx: Float,
    badgeHorizontalPaddingPx: Float,
    badgeVerticalPaddingPx: Float,
    badgeCornerRadiusPx: Float,
    textMeasurer: TextMeasurer,
) {
    val visible = referenceLines.zip(labels).mapNotNull { (line, label) ->
        val y = valueToY(line.price.value)
        if (y < plotTop || y > plotBottom) null else Triple(line, label, y)
    }
    visible.forEach { (line, _, y) ->
        drawLine(
            color = referenceColorByRole(line.kind),
            start = Offset(plotLeft, y),
            end = Offset(plotRight, y),
            strokeWidth = lineThicknessPx,
            pathEffect = lineDashEffect,
        )
    }
    var lastBadgeEndX = plotLeft + labelPaddingPx
    visible.forEach { (line, label, y) ->
        val measured = textMeasurer.measure(label, labelStyle)
        val badgeWidth = measured.size.width + 2f * badgeHorizontalPaddingPx
        val anchorX = if (line.overlapLevel == 0u) {
            plotLeft + labelPaddingPx
        } else {
            lastBadgeEndX + labelHorizontalGapPx
        }
        drawBadgeLabel(
            textMeasurer = textMeasurer,
            text = label,
            textStyle = labelStyle,
            backgroundColor = referenceColorByRole(line.kind),
            anchorX = anchorX,
            anchorY = y,
            horizontalPaddingPx = badgeHorizontalPaddingPx,
            verticalPaddingPx = badgeVerticalPaddingPx,
            cornerRadiusPx = badgeCornerRadiusPx,
        )
        lastBadgeEndX = anchorX + badgeWidth
    }
}

private fun DrawScope.drawCurrentPriceBadge(
    lastCandle: ChartCandleStick,
    lastTone: GemValueTone,
    priceLabel: String,
    valueToY: (Double) -> Float,
    plotTop: Float,
    plotBottom: Float,
    plotRight: Float,
    labelPaddingPx: Float,
    badgeHorizontalPaddingPx: Float,
    badgeVerticalPaddingPx: Float,
    badgeCornerRadiusPx: Float,
    labelStyle: TextStyle,
    upColor: Color,
    downColor: Color,
    flatColor: Color,
    textMeasurer: TextMeasurer,
) {
    val y = valueToY(lastCandle.close)
    if (y !in plotTop..plotBottom) return
    drawBadgeLabel(
        textMeasurer = textMeasurer,
        text = priceLabel,
        textStyle = labelStyle,
        backgroundColor = candleColor(lastTone, upColor, downColor, flatColor),
        anchorX = plotRight + labelPaddingPx,
        anchorY = y,
        horizontalPaddingPx = badgeHorizontalPaddingPx,
        verticalPaddingPx = badgeVerticalPaddingPx,
        cornerRadiusPx = badgeCornerRadiusPx,
    )
}

private fun DrawScope.drawSelection(
    selectedIndex: Int?,
    selectionAlpha: Float,
    candles: List<ChartCandleStick>,
    slotCenter: (Int) -> Float,
    valueToY: (Double) -> Float,
    plotTop: Float,
    plotBottom: Float,
    accentColor: Color,
    lineWidthPx: Float,
    dashLengthPx: Float,
    dotOuterRadiusPx: Float,
    dotBorderPx: Float,
) {
    if (selectedIndex == null || selectedIndex !in candles.indices || selectionAlpha <= 0f) return
    val centerX = slotCenter(selectedIndex)
    val closeY = valueToY(candles[selectedIndex].close)
    drawLine(
        color = accentColor.copy(alpha = CandlestickMetrics.SELECTION_LINE_ALPHA * selectionAlpha),
        start = Offset(centerX, plotTop),
        end = Offset(centerX, plotBottom),
        strokeWidth = lineWidthPx,
        pathEffect = PathEffect.dashPathEffect(floatArrayOf(dashLengthPx, dashLengthPx)),
    )
    val ringRadius = dotOuterRadiusPx - dotBorderPx / 2f
    drawCircle(
        color = Color.White.copy(alpha = selectionAlpha),
        radius = ringRadius,
        center = Offset(centerX, closeY),
    )
    drawCircle(
        color = accentColor.copy(alpha = selectionAlpha),
        radius = ringRadius,
        center = Offset(centerX, closeY),
        style = Stroke(width = dotBorderPx),
    )
}

private fun DrawScope.drawBadgeLabel(textMeasurer: TextMeasurer, text: String, textStyle: TextStyle, backgroundColor: Color, anchorX: Float, anchorY: Float, horizontalPaddingPx: Float, verticalPaddingPx: Float, cornerRadiusPx: Float) {
    val measured = textMeasurer.measure(text, textStyle)
    val badgeWidth = measured.size.width + horizontalPaddingPx * 2f
    val badgeHeight = measured.size.height + verticalPaddingPx * 2f
    val maxY = size.height - badgeHeight
    val topY = (anchorY - badgeHeight / 2f).coerceIn(0f, maxY)
    drawRoundRect(
        color = backgroundColor,
        topLeft = Offset(anchorX, topY),
        size = Size(badgeWidth, badgeHeight),
        cornerRadius = CornerRadius(cornerRadiusPx, cornerRadiusPx),
    )
    drawText(
        textLayoutResult = measured,
        topLeft = Offset(anchorX + horizontalPaddingPx, topY + verticalPaddingPx),
    )
}
