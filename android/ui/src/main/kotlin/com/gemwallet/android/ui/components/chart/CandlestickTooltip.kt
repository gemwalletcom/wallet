package com.gemwallet.android.ui.components.chart

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.layout.layoutId
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.color
import com.gemwallet.android.ui.models.chart.CandlestickTooltipUIModel
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space4
import com.gemwallet.android.ui.theme.space6

private object CandlestickTooltipMetrics {
    val ChipCornerRadius = 10.dp
    val BorderWidth = 1.dp
    val DividerThickness = 1.dp
    const val BackgroundAlpha = 0.92f
    const val BorderAlpha = 0.08f
    const val TabularNumbers = "tnum"
    const val DividerLayoutId = "tooltip-divider"
}

private data class TooltipCellData(
    val label: String,
    val value: String,
    val valueColor: Color,
)

@Composable
fun CandlestickTooltip(
    model: CandlestickTooltipUIModel,
    modifier: Modifier = Modifier,
) {
    val shape = RoundedCornerShape(CandlestickTooltipMetrics.ChipCornerRadius)
    val labelStyle = MaterialTheme.typography.labelSmall.copy(
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
    val valueStyle = MaterialTheme.typography.labelMedium.copy(
        fontWeight = FontWeight.Medium,
        fontFeatureSettings = CandlestickTooltipMetrics.TabularNumbers,
        textAlign = TextAlign.End,
    )
    val onSurface = MaterialTheme.colorScheme.onSurface

    val rows = listOf(
        TooltipCellData(stringResource(R.string.charts_price_open), model.open, onSurface),
        TooltipCellData(stringResource(R.string.charts_price_high), model.high, onSurface),
        TooltipCellData(stringResource(R.string.charts_price_low), model.low, onSurface),
        TooltipCellData(stringResource(R.string.charts_price_close), model.close, onSurface),
        TooltipCellData(stringResource(R.string.charts_price_change), model.changeText, model.changeDirection.color()),
        TooltipCellData(stringResource(R.string.perpetual_volume), model.volumeText, onSurface),
    )

    TooltipGrid(
        rows = rows,
        labelStyle = labelStyle,
        valueStyle = valueStyle,
        columnGap = paddingDefault,
        rowSpacing = space4,
        dividerAfter = 3,
        dividerColor = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f),
        dividerThickness = CandlestickTooltipMetrics.DividerThickness,
        dividerSpacing = space4,
        modifier = modifier
            .shadow(elevation = space6, shape = shape)
            .background(
                color = MaterialTheme.colorScheme.surfaceContainerHigh.copy(alpha = CandlestickTooltipMetrics.BackgroundAlpha),
                shape = shape,
            )
            .border(width = CandlestickTooltipMetrics.BorderWidth, color = Color.Black.copy(alpha = CandlestickTooltipMetrics.BorderAlpha), shape = shape)
            .padding(horizontal = paddingSmall, vertical = paddingSmall),
    )
}

@Composable
private fun TooltipGrid(
    rows: List<TooltipCellData>,
    labelStyle: TextStyle,
    valueStyle: TextStyle,
    columnGap: Dp,
    rowSpacing: Dp,
    dividerAfter: Int,
    dividerColor: Color,
    dividerThickness: Dp,
    dividerSpacing: Dp,
    modifier: Modifier = Modifier,
) {
    val density = LocalDensity.current
    val columnGapPx = with(density) { columnGap.roundToPx() }
    val rowSpacingPx = with(density) { rowSpacing.roundToPx() }
    val dividerSpacingPx = with(density) { dividerSpacing.roundToPx() }

    Layout(
        modifier = modifier,
        content = {
            rows.forEach { row ->
                Text(text = row.label, style = labelStyle)
                Text(text = row.value, style = valueStyle.copy(color = row.valueColor))
            }
            HorizontalDivider(
                modifier = Modifier.layoutId(CandlestickTooltipMetrics.DividerLayoutId),
                thickness = dividerThickness,
                color = dividerColor,
            )
        },
    ) { measurables, _ ->
        val cellMeasurables = measurables.filter { it.layoutId != CandlestickTooltipMetrics.DividerLayoutId }
        val dividerMeasurable = measurables.first { it.layoutId == CandlestickTooltipMetrics.DividerLayoutId }
        val unbounded = Constraints()
        val labels = List(rows.size) { cellMeasurables[it * 2].measure(unbounded) }
        val values = List(rows.size) { cellMeasurables[it * 2 + 1].measure(unbounded) }

        val labelColumnWidth = labels.maxOf { it.width }
        val valueColumnWidth = values.maxOf { it.width }
        val totalWidth = labelColumnWidth + columnGapPx + valueColumnWidth
        val divider = dividerMeasurable.measure(Constraints.fixedWidth(totalWidth))

        val rowHeights = List(rows.size) { maxOf(labels[it].height, values[it].height) }
        val showDivider = dividerAfter in 0 until rows.lastIndex
        val baseHeight = rowHeights.sum() + rowSpacingPx * (rows.size - 1)
        val totalHeight = if (showDivider) {
            baseHeight - rowSpacingPx + dividerSpacingPx + divider.height + dividerSpacingPx
        } else {
            baseHeight
        }

        layout(totalWidth, totalHeight) {
            var y = 0
            rows.indices.forEach { index ->
                val rowHeight = rowHeights[index]
                labels[index].place(0, y + (rowHeight - labels[index].height) / 2)
                values[index].place(totalWidth - values[index].width, y + (rowHeight - values[index].height) / 2)
                y += rowHeight
                when {
                    showDivider && index == dividerAfter -> {
                        y += dividerSpacingPx
                        divider.place(0, y)
                        y += divider.height + dividerSpacingPx
                    }
                    index < rows.lastIndex -> y += rowSpacingPx
                }
            }
        }
    }
}
