package com.gemwallet.android.ui.components.progress

import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ProgressIndicatorDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.space1
import com.gemwallet.android.ui.theme.space10
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.tinyIconSize

@Composable
fun CircularProgressIndicator10(
    modifier: Modifier = Modifier,
    color: Color = ProgressIndicatorDefaults.circularColor,
) {
    CircularProgressIndicator(
        modifier = modifier.size(size = space10),
        strokeWidth = space1,
        color = color,
    )
}

@Composable
fun CircularProgressIndicator14(
    modifier: Modifier = Modifier,
    color: Color = ProgressIndicatorDefaults.circularColor,
) {
    CircularProgressIndicator(
        modifier = modifier.size(size = 14.dp),
        strokeWidth = space1,
        color = color,
    )
}

@Composable
fun CircularProgressIndicator16(
    modifier: Modifier = Modifier,
    color: Color = ProgressIndicatorDefaults.circularColor,
) {
    CircularProgressIndicator(
        modifier = modifier.size(size = tinyIconSize),
        strokeWidth = space1,
        color = color,
    )
}

@Composable
fun CircularProgressIndicator20(
    modifier: Modifier = Modifier,
    color: Color = ProgressIndicatorDefaults.circularColor,
) {
    CircularProgressIndicator(
        modifier = modifier.size(size = compactIconSize),
        strokeWidth = space2,
        color = color,
    )
}