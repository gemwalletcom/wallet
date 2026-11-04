package com.gemwallet.android.ui.components.chart

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingSmall
import uniffi.gemstone.GemChartStyle

@Composable
internal fun ChartStyleButton(style: GemChartStyle, onClick: (GemChartStyle) -> Unit) {
    IconButton(
        onClick = { onClick(style) },
        modifier = Modifier.padding(end = paddingSmall).testTag("chart_style_switch"),
    ) {
        Icon(
            imageVector = when (style) {
                GemChartStyle.LINE -> AppIcons.ShowChart
                GemChartStyle.CANDLES -> AppIcons.CandlestickChart
            },
            contentDescription = stringResource(
                when (style) {
                    GemChartStyle.LINE -> R.string.charts_line
                    GemChartStyle.CANDLES -> R.string.charts_candlestick
                },
            ),
            tint = MaterialTheme.colorScheme.onSurface,
        )
    }
}
