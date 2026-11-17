package com.gemwallet.android.ui.components.chart

import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.icon
import uniffi.gemstone.GemChartStyle

@Composable
fun ChartStyleButton(style: GemChartStyle, onClick: (GemChartStyle) -> Unit) {
    IconButton(
        onClick = { onClick(style) },
        modifier = Modifier.testTag("chart_style_switch"),
    ) {
        Icon(
            imageVector = style.icon(),
            contentDescription = stringResource(style.stringRes()),
            tint = MaterialTheme.colorScheme.onSurface,
        )
    }
}
