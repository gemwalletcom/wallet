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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space10

private val statusIconSize = 18.dp
private val dismissButtonSize = 28.dp
private val dismissIconSize = 14.dp

@Composable
fun ConnectionStatusBanner(
    title: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    windowInsets: WindowInsets = WindowInsets(0.dp),
) {
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .testTag("connectionStatusBanner"),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
    ) {
        Row(
            modifier = Modifier
                .windowInsetsPadding(windowInsets)
                .padding(horizontal = padding16, vertical = space10),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                modifier = Modifier.size(statusIconSize),
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
                modifier = Modifier
                    .size(dismissButtonSize)
                    .testTag("connectionStatusDismiss"),
                onClick = onDismiss,
            ) {
                Icon(
                    modifier = Modifier.size(dismissIconSize),
                    imageVector = AppIcons.Close,
                    tint = MaterialTheme.colorScheme.secondary,
                    contentDescription = "dismiss",
                )
            }
        }
    }
}
