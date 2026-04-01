package com.gemwallet.android.ui.components.list_item

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.foundation.layout.size
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.theme.compactIconSize

@Composable
fun SelectionCheckmark(
    modifier: Modifier = Modifier,
) {
    Icon(
        modifier = modifier.size(compactIconSize),
        imageVector = Icons.Default.CheckCircle,
        contentDescription = null,
        tint = MaterialTheme.colorScheme.primary,
    )
}
