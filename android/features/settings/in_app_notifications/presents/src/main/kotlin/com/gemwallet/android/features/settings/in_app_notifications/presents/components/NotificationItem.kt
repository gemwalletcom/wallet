package com.gemwallet.android.features.settings.in_app_notifications.presents.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.list_item.ListItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.alpha10
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space6
import com.gemwallet.android.ui.theme.space8
import com.wallet.core.primitives.CoreEmoji
import com.wallet.core.primitives.CoreListItemIcon
import com.wallet.core.primitives.InAppNotification

@Composable
fun NotificationItem(
    notification: InAppNotification,
    listPosition: ListPosition,
    onOpenUrl: (String) -> Unit,
) {
    val item = notification.item
    val icon = item.icon
    val url = item.url
    val subtitle = item.subtitle
    val value = item.value
    val subvalue = item.subvalue
    ListItem(
        modifier = if (url != null) Modifier.clickable { onOpenUrl(url) } else Modifier,
        listPosition = listPosition,
        leading = if (icon != null) {
            { NotificationIcon(icon) }
        } else {
            null
        },
        title = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    modifier = Modifier.weight(1f, fill = false),
                    text = item.title,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (notification.readAt == null) {
                    Spacer(Modifier.width(space8))
                    NewBadge()
                }
            }
        },
        subtitle = if (subtitle != null) {
            {
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.secondary,
                )
            }
        } else {
            null
        },
        trailing = {
            DataBadgeChevron(isShowChevron = url != null) {
                if (value != null || subvalue != null) {
                    Column(horizontalAlignment = Alignment.End) {
                        if (value != null) {
                            Text(
                                text = value,
                                style = MaterialTheme.typography.bodyLarge,
                                fontWeight = FontWeight.SemiBold,
                                color = MaterialTheme.colorScheme.onSurface,
                                maxLines = 1,
                            )
                        }
                        if (subvalue != null) {
                            Text(
                                text = subvalue,
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.secondary,
                                maxLines = 1,
                            )
                        }
                    }
                }
            }
        },
    )
}

@Composable
private fun NewBadge() {
    Text(
        modifier = Modifier
            .background(
                color = MaterialTheme.colorScheme.primary.copy(alpha = alpha10),
                shape = RoundedCornerShape(space6),
            )
            .padding(horizontal = space6, vertical = space2),
        text = stringResource(R.string.assets_tags_new),
        color = MaterialTheme.colorScheme.primary,
        style = MaterialTheme.typography.bodyMedium,
    )
}

@Composable
private fun NotificationIcon(icon: CoreListItemIcon) {
    when (icon) {
        is CoreListItemIcon.Emoji -> Box(
            modifier = Modifier.size(listItemIconSize),
            contentAlignment = Alignment.Center,
        ) {
            val emojiSize = with(LocalDensity.current) { (listItemIconSize * EmojiSizeRatio).toSp() }
            Text(
                text = icon.value.glyph(),
                fontSize = emojiSize,
                lineHeight = emojiSize,
                textAlign = TextAlign.Center,
            )
        }
        is CoreListItemIcon.Image -> AsyncImage(model = icon.value, size = listItemIconSize)
        is CoreListItemIcon.Asset -> AsyncImage(model = icon.value.getIconUrl(), size = listItemIconSize)
    }
}

private fun CoreEmoji.glyph(): String = when (this) {
    CoreEmoji.Gift -> "🎁"
    CoreEmoji.Gem -> "💎"
    CoreEmoji.Party -> "🎉"
    CoreEmoji.Warning -> "⚠️"
}

private const val EmojiSizeRatio = 0.75f
