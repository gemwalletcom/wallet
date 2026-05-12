package com.gemwallet.android.ui.components.list_item

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import com.gemwallet.android.domains.price.ValueDirection

@Composable
fun ValueDirection.color(): Color {
    return when (this) {
        ValueDirection.Up -> MaterialTheme.colorScheme.tertiary
        ValueDirection.Down -> MaterialTheme.colorScheme.error
        ValueDirection.None -> MaterialTheme.colorScheme.secondary
    }
}
