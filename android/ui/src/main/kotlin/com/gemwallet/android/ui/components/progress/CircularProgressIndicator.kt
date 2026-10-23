package com.gemwallet.android.ui.components.progress

import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ProgressIndicatorDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.space0
import com.gemwallet.android.ui.theme.space1
import com.gemwallet.android.ui.theme.space10
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.tinyIconSize

private val progressSize14 = 14.dp

@Composable
fun CircularProgressIndicator10(modifier: Modifier = Modifier, color: Color = ProgressIndicatorDefaults.circularColor) {
    CircularProgressIndicator(
        modifier = modifier.size(size = space10),
        strokeWidth = space1,
        color = color,
    )
}

@Composable
fun CircularProgressIndicator14(modifier: Modifier = Modifier, color: Color = ProgressIndicatorDefaults.circularColor) {
    CircularProgressIndicator(
        modifier = modifier.size(size = progressSize14),
        strokeWidth = space1,
        color = color,
    )
}

@Composable
fun CircularProgressIndicator14(progress: () -> Float, color: Color, trackColor: Color, modifier: Modifier = Modifier) {
    CircularProgressIndicator(
        progress = progress,
        modifier = modifier.size(size = progressSize14),
        color = color,
        strokeWidth = space2,
        trackColor = trackColor,
        strokeCap = StrokeCap.Round,
        gapSize = space0,
    )
}

@Composable
fun CircularProgressIndicator16(modifier: Modifier = Modifier, color: Color = ProgressIndicatorDefaults.circularColor) {
    CircularProgressIndicator(
        modifier = modifier.size(size = tinyIconSize),
        strokeWidth = space1,
        color = color,
    )
}

@Composable
fun CircularProgressIndicator20(modifier: Modifier = Modifier, color: Color = ProgressIndicatorDefaults.circularColor) {
    CircularProgressIndicator(
        modifier = modifier.size(size = compactIconSize),
        strokeWidth = space2,
        color = color,
    )
}
