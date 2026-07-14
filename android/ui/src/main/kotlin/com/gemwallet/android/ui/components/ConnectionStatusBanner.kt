package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.pendingColor

@Composable
fun ConnectionStatusBanner(
    title: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    windowInsets: WindowInsets = WindowInsets(0.dp),
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
    ) {
        Row(
            modifier = Modifier
                .windowInsetsPadding(windowInsets)
                .padding(horizontal = padding16, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                modifier = Modifier.size(18.dp),
                imageVector = AppIcons.Warning,
                tint = pendingColor,
                contentDescription = null,
            )
            Text(
                modifier = Modifier
                    .weight(1f)
                    .padding(horizontal = paddingSmall),
                text = title,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
            IconButton(
                modifier = Modifier.size(28.dp),
                onClick = onDismiss,
            ) {
                Icon(
                    modifier = Modifier.size(14.dp),
                    imageVector = AppIcons.Close,
                    tint = MaterialTheme.colorScheme.secondary,
                    contentDescription = "dismiss",
                )
            }
        }
    }
}
