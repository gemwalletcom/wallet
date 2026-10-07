package com.gemwallet.android.ui.components.chart

import android.text.format.DateFormat
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
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
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
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space1
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space24
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8
import uniffi.gemstone.ChartCandleStick
import uniffi.gemstone.GemCandleChart
import uniffi.gemstone.GemCandleTick
import uniffi.gemstone.GemCandleTickFormat
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemPerpetualChartLine
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemValueTone
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale
import kotlin.math.min

private object CandlestickMetrics {
    val topPadding = space4
    val bottomPadding = space24
    val leftPadding = space8
    val rightPadding = space8
    val labelPadding = space4
    val volumeBandGap = space4
    val timeLabelGap = space4
    val minWickWidth = space1
    val maxWickWidth = space2
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
    const val BODY_CORNER_RATIO = 0.15f
    const val VOLUME_ALPHA = 0.12f
    const val TEXT_CACHE_SIZE = 32
}

@Composable
fun GemCandlestickChart(chart: GemCandleChart, onZoom: (Float, Float) -> Unit, onPan: (Float) -> Unit, selectedIndex: Int? = null, onSelectionChanged: (Int?) -> Unit = {}, modifier: Modifier = Modifier) {
    if (chart.candles.isEmpty()) return

    val layout = chart.layout
    val context = LocalContext.current
    val lineLabels = remember(chart) { layout.lines.map { it.label.string(context) } }
    val timeLabels = remember(chart) { chartTimeLabels(chart.xTicks, ZoneId.systemDefault(), Locale.getDefault()) }
    val density = LocalDensity.current
    val textMeasurer = rememberTextMeasurer(cacheSize = CandlestickMetrics.TEXT_CACHE_SIZE)

    val upColor = ListItemTextStyle.Positive.color()
    val downColor = ListItemTextStyle.Negative.color()
    val flatColor = ListItemTextStyle.Secondary.color()
    val axisLabelColor = MaterialTheme.colorScheme.secondary
    val gridGuidelineColor = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.13f)
    val volumeColor = MaterialTheme.colorScheme.onSurface.copy(alpha = CandlestickMetrics.VOLUME_ALPHA)
    val selectionAccentColor = MaterialTheme.colorScheme.primary
    val currentPriceLineColor = MaterialTheme.colorScheme.secondary.copy(alpha = 0.6f)
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

    val minWickWidthPx = with(density) { CandlestickMetrics.minWickWidth.toPx() }
    val maxWickWidthPx = with(density) { CandlestickMetrics.maxWickWidth.toPx() }
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
    val priceTextWidthPx = remember(layout.levels, layout.currentPrice, axisLabelStyle, density) {
        val levelWidths = layout.levels.map { textMeasurer.measure(it.text(), axisLabelStyle).size.width }
        (levelWidths + textMeasurer.measure(layout.currentPrice.text(), whiteLabelStyle).size.width).max().toFloat()
    }
    val rightAxisWidthPx = labelPaddingPx + priceTextWidthPx + with(density) { CandlestickMetrics.rightPadding.toPx() }
    val volumeBandGapPx = with(density) { CandlestickMetrics.volumeBandGap.toPx() }

    var chartSize by remember { mutableStateOf(IntSize.Zero) }
    val selection = rememberChartSelection(selectedIndex)
    val priceRange = rememberChartRange(layout.priceLow.toFloat(), layout.priceHigh.toFloat())
    val volumeRange = rememberChartRange(0f, layout.volumeHigh.toFloat())

    Box(modifier = modifier.fillMaxSize().onSizeChanged { chartSize = it }) {
        if (chartSize.width <= 0 || chartSize.height <= 0) return@Box

        val frame = Rect(leftPaddingPx, topPaddingPx, chartSize.width - rightAxisWidthPx, chartSize.height - bottomPaddingPx)
        if (frame.width <= 0 || frame.height <= 0) return@Box
        val priceViewport = priceRange.viewport ?: return@Box
        val plot = CandlestickPlot(chart, frame, volumeBandGapPx, minWickWidthPx, maxWickWidthPx, priceViewport, volumeRange.viewport)

        val candleIndex by rememberUpdatedState { fraction: Float -> chart.indexAt(fraction.toDouble())?.toInt() }
        val selectionChanged by rememberUpdatedState(onSelectionChanged)
        val zoom by rememberUpdatedState(onZoom)
        val pan by rememberUpdatedState(onPan)
        val zoomed by rememberUpdatedState(chart.isZoomed)

        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .chartGestures(
                    plotLeft = frame.left,
                    plotWidth = frame.width,
                    isZoomed = { zoomed },
                    indexAt = { candleIndex(it) },
                    onSelectionChanged = { selectionChanged(it) },
                    onZoom = { magnification, anchor -> zoom(magnification, anchor) },
                    onPan = { pan(it) },
                ),
        ) {
            drawYAxis(layout.levels, plot, gridGuidelineColor, gridDashEffect, textMeasurer, axisLabelStyle, labelPaddingPx)
            drawTimeAxis(chart.xTicks.map { plot.x(it.date) }, timeLabels, plot, gridGuidelineColor, gridDashEffect, textMeasurer, timeLabelStyle, timeLabelGapPx)
            drawCurrentPriceLine(layout.currentPrice.value, plot, currentPriceLineColor, currentPriceDashEffect)
            clipRect(left = frame.left, top = 0f, right = frame.right, bottom = size.height) {
                drawVolumes(chart.candles, plot, volumeColor)
                drawCandles(chart.candles, layout.tones, plot, upColor, downColor, flatColor)
            }
            drawReferenceLines(
                referenceLines = layout.lines,
                labels = lineLabels,
                referenceColorByRole = referenceColorByRole,
                plot = plot,
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
                price = layout.currentPrice.value,
                tone = layout.currentTone,
                priceLabel = layout.currentPrice.text(),
                plot = plot,
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
                plot = plot,
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

private fun DrawScope.drawYAxis(ticks: List<GemFormattedNumber>, plot: CandlestickPlot, guidelineColor: Color, guidelineDash: PathEffect, textMeasurer: TextMeasurer, style: TextStyle, labelPaddingPx: Float) {
    ticks.forEach { tick ->
        val y = plot.y(tick.value)
        drawLine(
            color = guidelineColor,
            start = Offset(plot.frame.left, y),
            end = Offset(plot.frame.right, y),
            strokeWidth = 1f,
            pathEffect = guidelineDash,
        )
        val measured = textMeasurer.measure(tick.text(), style)
        drawText(textLayoutResult = measured, topLeft = Offset(plot.frame.right + labelPaddingPx, y - measured.size.height / 2f))
    }
}

private fun DrawScope.drawTimeAxis(positions: List<Float>, labels: List<String>, plot: CandlestickPlot, color: Color, dash: PathEffect, textMeasurer: TextMeasurer, style: TextStyle, labelGapPx: Float) {
    positions.zip(labels).forEach { (x, label) ->
        drawLine(
            color = color,
            start = Offset(x, plot.frame.top),
            end = Offset(x, plot.frame.bottom),
            strokeWidth = 1f,
            pathEffect = dash,
        )
        val measured = textMeasurer.measure(label, style)
        drawText(textLayoutResult = measured, topLeft = Offset(x - measured.size.width / 2f, plot.frame.bottom + labelGapPx))
    }
}

private fun DrawScope.drawCurrentPriceLine(price: Double, plot: CandlestickPlot, color: Color, dash: PathEffect) {
    val y = plot.y(price)
    if (y !in plot.frame.top..plot.priceBottom) return
    drawLine(
        color = color,
        start = Offset(plot.frame.left, y),
        end = Offset(plot.frame.right, y),
        strokeWidth = 1f,
        pathEffect = dash,
    )
}

private fun DrawScope.drawVolumes(candles: List<ChartCandleStick>, plot: CandlestickPlot, color: Color) {
    candles.filter { it.volume > 0.0 }.mapNotNull(plot::volume).forEach { bar ->
        val corner = min(plot.bodyWidth * CandlestickMetrics.BODY_CORNER_RATIO, bar.height / 2f)
        drawRoundRect(color = color, topLeft = bar.topLeft, size = bar.size, cornerRadius = CornerRadius(corner, corner))
    }
}

private fun DrawScope.drawCandles(candles: List<ChartCandleStick>, tones: List<GemValueTone>, plot: CandlestickPlot, upColor: Color, downColor: Color, flatColor: Color) {
    candles.zip(tones).forEach { (candle, tone) ->
        val color = candleColor(tone, upColor, downColor, flatColor)
        val wick = plot.wick(candle)
        drawLine(color = color, start = wick.topCenter, end = wick.bottomCenter, strokeWidth = wick.width, cap = StrokeCap.Round)
        val body = plot.body(candle)
        val corner = min(plot.bodyWidth * CandlestickMetrics.BODY_CORNER_RATIO, body.height / 2f)
        drawRoundRect(color = color, topLeft = body.topLeft, size = body.size, cornerRadius = CornerRadius(corner, corner))
    }
}

private fun DrawScope.drawReferenceLines(
    referenceLines: List<GemPerpetualChartLine>,
    labels: List<String>,
    referenceColorByRole: (GemPerpetualChartLineKind) -> Color,
    plot: CandlestickPlot,
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
        val y = plot.y(line.price.value)
        if (y !in plot.frame.top..plot.priceBottom) null else Triple(line, label, y)
    }
    visible.forEach { (line, _, y) ->
        drawLine(
            color = referenceColorByRole(line.kind),
            start = Offset(plot.frame.left, y),
            end = Offset(plot.frame.right, y),
            strokeWidth = lineThicknessPx,
            pathEffect = lineDashEffect,
        )
    }
    var lastBadgeEndX = plot.frame.left + labelPaddingPx
    visible.forEach { (line, label, y) ->
        val measured = textMeasurer.measure(label, labelStyle)
        val badgeWidth = measured.size.width + 2f * badgeHorizontalPaddingPx
        val anchorX = if (line.overlapLevel == 0u) {
            plot.frame.left + labelPaddingPx
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
    price: Double,
    tone: GemValueTone,
    priceLabel: String,
    plot: CandlestickPlot,
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
    val y = plot.y(price)
    if (y !in plot.frame.top..plot.priceBottom) return
    drawBadgeLabel(
        textMeasurer = textMeasurer,
        text = priceLabel,
        textStyle = labelStyle,
        backgroundColor = candleColor(tone, upColor, downColor, flatColor),
        anchorX = plot.frame.right + labelPaddingPx - badgeHorizontalPaddingPx,
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
    plot: CandlestickPlot,
    accentColor: Color,
    lineWidthPx: Float,
    dashLengthPx: Float,
    dotOuterRadiusPx: Float,
    dotBorderPx: Float,
) {
    if (selectedIndex == null || selectedIndex !in candles.indices || selectionAlpha <= 0f) return
    val centerX = plot.x(candles[selectedIndex].date)
    val closeY = plot.y(candles[selectedIndex].close)
    drawLine(
        color = accentColor.copy(alpha = CandlestickMetrics.SELECTION_LINE_ALPHA * selectionAlpha),
        start = Offset(centerX, plot.frame.top),
        end = Offset(centerX, plot.frame.bottom),
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

internal fun chartTimeLabels(ticks: List<GemCandleTick>, zone: ZoneId, locale: Locale): List<String> {
    val time = DateTimeFormatter.ofLocalizedTime(FormatStyle.SHORT).withLocale(locale)
    val day by lazy { bestPattern("dMMM", locale) }
    val monthYear by lazy { bestPattern("MMMy", locale) }
    return ticks.map { tick ->
        when (tick.format) {
            GemCandleTickFormat.TIME -> time
            GemCandleTickFormat.DAY -> day
            GemCandleTickFormat.MONTH_YEAR -> monthYear
        }.format(Instant.ofEpochMilli(tick.date).atZone(zone))
    }
}

private fun bestPattern(skeleton: String, locale: Locale): DateTimeFormatter = DateTimeFormatter.ofPattern(DateFormat.getBestDateTimePattern(locale, skeleton), locale)
