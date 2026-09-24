package com.gemwallet.android.ui.components.chart

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.models.chart.CandleDirection
import com.gemwallet.android.ui.models.chart.CandleUIModel
import com.gemwallet.android.ui.models.chart.CandlestickChartUIModel
import com.gemwallet.android.ui.models.chart.ChartAxisTick
import com.gemwallet.android.ui.models.chart.ChartReferenceLineUIModel
import com.gemwallet.android.ui.models.chart.ChartTimeTick
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space1
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space24
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8
import uniffi.gemstone.GemPerpetualChartLineKind
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

private object CandlestickMetrics {
    val topPadding = paddingDefault
    val bottomPadding = space24
    val rightAxisWidth = 88.dp
    val leftPadding = space8
    val labelPadding = space4
    val timeLabelGap = space4
    val wickWidth = space1
    val currentPriceDash = space2
    val currentPriceGap = 3.dp
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
fun GemCandlestickChart(model: CandlestickChartUIModel, selectedIndex: Int? = null, onSelectionChanged: (Int?) -> Unit = {}, onZoom: (Float) -> Unit = {}, modifier: Modifier = Modifier) {
    if (model.candles.isEmpty()) return

    val density = LocalDensity.current
    val textMeasurer = rememberTextMeasurer()

    val upColor = ListItemTextStyle.Positive.color()
    val downColor = ListItemTextStyle.Negative.color()
    val flatColor = ListItemTextStyle.Secondary.color()
    val axisLabelColor = MaterialTheme.colorScheme.secondary
    val gridGuidelineColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.13f)
    val currentPriceLineColor = MaterialTheme.colorScheme.secondary.copy(alpha = 0.6f)
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
    val timeLabelStyle = remember(axisLabelStyle) { axisLabelStyle.copy(textAlign = TextAlign.Center) }
    val currentPriceDashEffect = remember(density) {
        with(density) { PathEffect.dashPathEffect(floatArrayOf(CandlestickMetrics.currentPriceDash.toPx(), CandlestickMetrics.currentPriceGap.toPx())) }
    }

    val wickWidthPx = with(density) { CandlestickMetrics.wickWidth.toPx() }
    val timeLabelGapPx = with(density) { CandlestickMetrics.timeLabelGap.toPx() }
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

        val bodyWidth = max(1f, model.bodyWidthFraction * plotWidth)

        fun candleX(index: Int): Float = plotLeft + model.candles[index].x * plotWidth
        fun valueToY(value: Double): Float = (plotBottom - (value - model.yMin) / model.ySpan * plotHeight).toFloat()

        val indexAt by rememberUpdatedState { x: Float ->
            model.candles.indices.takeIf { x in plotLeft..plotRight }?.minByOrNull { abs(candleX(it) - x) }
        }
        val selectionChanged by rememberUpdatedState(onSelectionChanged)
        val zoom by rememberUpdatedState(onZoom)

        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .chartGestures(
                    indexAt = { indexAt(it) },
                    onSelectionChanged = { selectionChanged(it) },
                    onZoom = { zoom(it) },
                ),
        ) {
            drawYAxis(model.yTicks, plotLeft, plotRight, plotTop, plotBottom, gridGuidelineColor, gridDashEffect, textMeasurer, axisLabelStyle, labelPaddingPx)
            drawTimeAxis(model.xTicks, plotLeft, plotWidth, plotTop, plotBottom, gridGuidelineColor, gridDashEffect, textMeasurer, timeLabelStyle, timeLabelGapPx)
            drawCurrentPriceLine(model.candles.last(), ::valueToY, plotLeft, plotRight, plotTop, plotBottom, currentPriceLineColor, currentPriceDashEffect)
            clipRect(left = plotLeft, top = 0f, right = plotRight, bottom = size.height) {
                drawCandles(model.candles, ::candleX, ::valueToY, bodyWidth, wickWidthPx, upColor, downColor, flatColor)
            }
            drawReferenceLines(
                referenceLines = model.referenceLines,
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
                lastCandle = model.candles.last(),
                priceLabel = model.currentPriceLabel,
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
                candles = model.candles,
                candleX = ::candleX,
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

private fun candleColor(candle: CandleUIModel, up: Color, down: Color, flat: Color): Color = when (candle.direction) {
    CandleDirection.Up -> up
    CandleDirection.Down -> down
    CandleDirection.Flat -> flat
}

@Composable
private fun GemPerpetualChartLineKind.color(): Color = when (this) {
    GemPerpetualChartLineKind.ENTRY -> MaterialTheme.colorScheme.outline
    GemPerpetualChartLineKind.LIQUIDATION -> MaterialTheme.colorScheme.error
    GemPerpetualChartLineKind.STOP_LOSS -> pendingColor
    GemPerpetualChartLineKind.TAKE_PROFIT -> MaterialTheme.colorScheme.tertiary
}

private fun DrawScope.drawYAxis(
    ticks: List<ChartAxisTick>,
    plotLeft: Float,
    plotRight: Float,
    plotTop: Float,
    plotBottom: Float,
    guidelineColor: Color,
    guidelineDash: PathEffect,
    textMeasurer: TextMeasurer,
    style: TextStyle,
    labelPaddingPx: Float,
) {
    val plotHeight = plotBottom - plotTop
    ticks.forEach { tick ->
        val y = plotBottom - plotHeight * tick.fraction
        drawLine(
            color = guidelineColor,
            start = Offset(plotLeft, y),
            end = Offset(plotRight, y),
            strokeWidth = 1f,
            pathEffect = guidelineDash,
        )
        val measured = textMeasurer.measure(tick.label, style)
        drawText(textLayoutResult = measured, topLeft = Offset(plotRight + labelPaddingPx, y - measured.size.height / 2f))
    }
}

private fun DrawScope.drawTimeAxis(ticks: List<ChartTimeTick>, plotLeft: Float, plotWidth: Float, plotTop: Float, plotBottom: Float, color: Color, dash: PathEffect, textMeasurer: TextMeasurer, style: TextStyle, labelGapPx: Float) {
    ticks.forEach { tick ->
        val x = plotLeft + plotWidth * tick.fraction
        drawLine(
            color = color,
            start = Offset(x, plotTop),
            end = Offset(x, plotBottom),
            strokeWidth = 1f,
            pathEffect = dash,
        )
        val measured = textMeasurer.measure(tick.label, style)
        drawText(textLayoutResult = measured, topLeft = Offset(x - measured.size.width / 2f, plotBottom + labelGapPx))
    }
}

private fun DrawScope.drawCurrentPriceLine(lastCandle: CandleUIModel, valueToY: (Double) -> Float, plotLeft: Float, plotRight: Float, plotTop: Float, plotBottom: Float, color: Color, dash: PathEffect) {
    val y = valueToY(lastCandle.close)
    if (y !in plotTop..plotBottom) return
    drawLine(
        color = color,
        start = Offset(plotLeft, y),
        end = Offset(plotRight, y),
        strokeWidth = 1f,
        pathEffect = dash,
    )
}

private fun DrawScope.drawCandles(candles: List<CandleUIModel>, candleX: (Int) -> Float, valueToY: (Double) -> Float, bodyWidth: Float, wickWidthPx: Float, upColor: Color, downColor: Color, flatColor: Color) {
    candles.forEachIndexed { index, candle ->
        val color = candleColor(candle, upColor, downColor, flatColor)
        val centerX = candleX(index)
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
    referenceLines: List<ChartReferenceLineUIModel>,
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
    val visible = referenceLines.mapNotNull { line ->
        val y = valueToY(line.price)
        if (y < plotTop || y > plotBottom) null else line to y
    }
    visible.forEach { (line, y) ->
        drawLine(
            color = referenceColorByRole(line.kind),
            start = Offset(plotLeft, y),
            end = Offset(plotRight, y),
            strokeWidth = lineThicknessPx,
            pathEffect = lineDashEffect,
        )
    }
    var lastBadgeEndX = plotLeft + labelPaddingPx
    visible.forEach { (line, y) ->
        val measured = textMeasurer.measure(line.label, labelStyle)
        val badgeWidth = measured.size.width + 2f * badgeHorizontalPaddingPx
        val anchorX = if (line.overlapLevel == 0) {
            plotLeft + labelPaddingPx
        } else {
            lastBadgeEndX + labelHorizontalGapPx
        }
        drawBadgeLabel(
            textMeasurer = textMeasurer,
            text = line.label,
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
    lastCandle: CandleUIModel,
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
        backgroundColor = candleColor(lastCandle, upColor, downColor, flatColor),
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
    candles: List<CandleUIModel>,
    candleX: (Int) -> Float,
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
    val centerX = candleX(selectedIndex)
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
