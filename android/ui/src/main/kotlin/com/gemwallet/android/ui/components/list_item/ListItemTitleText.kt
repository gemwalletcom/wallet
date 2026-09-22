package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.theme.paddingHalfSmall

@Composable
fun ListItemTitleText(text: String, titleBadge: (@Composable () -> Unit)? = null, color: Color = MaterialTheme.colorScheme.onSurface, style: TextStyle = MaterialTheme.typography.titleMedium, maxLines: Int = 1) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(paddingHalfSmall),
    ) {
        Text(
            modifier = Modifier.weight(1f, false),
            text = text,
            maxLines = maxLines,
            overflow = if (maxLines == 1) TextOverflow.MiddleEllipsis else TextOverflow.Ellipsis,
            style = style,
            color = color,
        )
        if (titleBadge != null) {
            Box(
                modifier = Modifier
                    .height(0.dp)
                    .wrapContentHeight(Alignment.CenterVertically, unbounded = true),
            ) {
                titleBadge()
            }
        }
    }
}
