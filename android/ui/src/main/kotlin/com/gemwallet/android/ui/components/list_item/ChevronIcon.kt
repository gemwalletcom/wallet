package com.gemwallet.android.ui.components.list_item

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.foundation.layout.size
import com.gemwallet.android.ui.theme.tinyIconSize

@Composable
fun ChevronIcon(
    modifier: Modifier = Modifier,
    size: Dp = tinyIconSize,
    tint: Color = MaterialTheme.colorScheme.secondary,
) {
    Icon(
        imageVector = Icons.AutoMirrored.Filled.KeyboardArrowRight,
        contentDescription = null,
        modifier = modifier.size(size),
        tint = tint,
    )
}
