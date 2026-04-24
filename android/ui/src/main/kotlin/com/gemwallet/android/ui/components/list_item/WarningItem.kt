package com.gemwallet.android.ui.components.list_item

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.Spacer4
import com.gemwallet.android.ui.theme.Spacer8
import com.gemwallet.android.ui.theme.defaultPadding
import com.gemwallet.android.ui.theme.smallIconSize
import com.gemwallet.android.ui.theme.alpha50

@Composable
fun WarningItem(
    title: String,
    message: String? = null,
    color: Color,
    position: ListPosition,
    onClick: (() -> Unit)? = null,
    trailing: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier
            .listItem(position)
            .fillMaxWidth()
            .defaultPadding()
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    modifier = Modifier.size(smallIconSize),
                    imageVector = Icons.Outlined.Warning,
                    contentDescription = null,
                    tint = color,
                )
                Spacer8()
                Text(
                    text = title,
                    color = color,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            message?.takeIf { it.isNotBlank() }?.let {
                Spacer4()
                Row(verticalAlignment = Alignment.CenterVertically) {
                    trailing?.invoke()
                    Text(
                        text = it,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.secondary,
                    )
                }
            }
        }
        if (onClick != null) {
            ChevronIcon(tint = MaterialTheme.colorScheme.secondary.copy(alpha = alpha50))
        }
    }
}
