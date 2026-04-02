package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.asset.getSupportIconUrl
import com.gemwallet.android.ui.theme.listItemIconSize
import com.wallet.core.primitives.Asset

@Composable
fun AssetIcon(
    asset: Asset,
    size: Dp = listItemIconSize,
) {
    IconWithBadge(
        icon = asset.getIconUrl(),
        placeholder = asset.type.string,
        supportIcon = asset.getSupportIconUrl(),
        size = size,
    )
}

@Composable
fun IconWithBadge(
    icon: Any?,
    placeholder: String? = null,
    supportIcon: Any? = null,
    size: Dp = listItemIconSize,
) {
    icon ?: return
    BadgedIcon(
        icon = icon,
        placeholder = placeholder,
        size = size,
        badge = supportIcon?.let { url -> { AsyncImage(model = url, contentDescription = "list_item_support_icon") } },
    )
}

@Composable
fun IconWithBadge(
    icon: Any?,
    placeholder: String? = null,
    size: Dp = listItemIconSize,
    badge: @Composable () -> Unit,
) {
    icon ?: return
    BadgedIcon(icon = icon, placeholder = placeholder, size = size, badge = badge)
}

@Composable
fun IconWithBadge(
    size: Dp = listItemIconSize,
    badge: (@Composable () -> Unit)? = null,
    content: @Composable () -> Unit,
) {
    BadgedBox(size = size, badge = badge, content = content)
}

private const val BADGE_SIZE_RATIO = 2.6f
private val BADGE_BORDER_WIDTH = 2.dp

@Composable
private fun BadgedIcon(
    icon: Any,
    placeholder: String?,
    size: Dp,
    badge: (@Composable () -> Unit)? = null,
) {
    BadgedBox(size = size, badge = badge) {
        AsyncImage(
            model = icon,
            placeholderText = placeholder,
            contentDescription = "list_item_icon",
            size = size,
        )
    }
}

@Composable
private fun BadgedBox(
    size: Dp,
    badge: (@Composable () -> Unit)?,
    content: @Composable () -> Unit,
) {
    Box {
        content()
        if (badge != null) {
            val badgeSize = size / BADGE_SIZE_RATIO + BADGE_BORDER_WIDTH * 2
            val badgeOffset = badgeSize / 4
            Box(
                modifier = Modifier
                    .offset(badgeOffset, badgeOffset)
                    .size(badgeSize)
                    .align(Alignment.BottomEnd)
                    .border(BADGE_BORDER_WIDTH, MaterialTheme.colorScheme.surface, CircleShape),
            ) {
                badge()
            }
        }
    }
}
