package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.theme.compactIconSize

@Composable
fun SelectionCheckmark(
    modifier: Modifier = Modifier,
    size: Dp = compactIconSize,
    color: Color = MaterialTheme.colorScheme.primary,
) {
    Box(
        modifier = modifier
            .size(size)
            .background(color, CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Canvas(modifier = Modifier.size(size / 1.55f)) {
            val strokeWidth = this.size.minDimension * 0.145f
            drawLine(
                color = Color.White,
                start = Offset(x = this.size.width * 0.18f, y = this.size.height * 0.52f),
                end = Offset(x = this.size.width * 0.42f, y = this.size.height * 0.74f),
                strokeWidth = strokeWidth,
                cap = StrokeCap.Round,
            )
            drawLine(
                color = Color.White,
                start = Offset(x = this.size.width * 0.42f, y = this.size.height * 0.74f),
                end = Offset(x = this.size.width * 0.82f, y = this.size.height * 0.28f),
                strokeWidth = strokeWidth,
                cap = StrokeCap.Round,
            )
        }
    }
}
