package com.gemwallet.android.ui.components.list_item

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow

@Composable
fun ListItemSupportText(text: String, color: Color = MaterialTheme.colorScheme.secondary, maxLines: Int = 1) {
    Text(
        modifier = Modifier,
        text = text,
        maxLines = maxLines,
        overflow = TextOverflow.Ellipsis,
        color = color,
        style = MaterialTheme.typography.bodyMedium,
    )
}
