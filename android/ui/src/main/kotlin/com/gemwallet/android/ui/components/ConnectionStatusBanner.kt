package com.gemwallet.android.ui.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.compactIconSize
import com.gemwallet.android.ui.theme.iconSize
import com.gemwallet.android.ui.theme.padding16
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.pendingColor
import com.gemwallet.android.ui.theme.space10
import com.gemwallet.android.ui.theme.tinyIconSize

@Composable
fun ConnectionStatusBanner(
    title: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .testTag("connectionStatusBanner"),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = padding16, vertical = space10),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                modifier = Modifier.size(compactIconSize),
                imageVector = AppIcons.Warning,
                tint = pendingColor,
                contentDescription = null,
            )
            Column(
                modifier = Modifier
                    .weight(1f)
                    .padding(horizontal = paddingSmall),
            ) {
                Text(
                    text = title,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    text = stringResource(R.string.errors_balances_activity_outdated),
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.secondary,
                )
            }
            IconButton(
                modifier = Modifier
                    .size(iconSize)
                    .testTag("connectionStatusDismiss"),
                onClick = onDismiss,
            ) {
                Icon(
                    modifier = Modifier.size(tinyIconSize),
                    imageVector = AppIcons.Close,
                    tint = MaterialTheme.colorScheme.secondary,
                    contentDescription = "dismiss",
                )
            }
        }
    }
}
