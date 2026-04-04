package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
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
    IconWithBadge(
        size = size,
        badge = supportIcon?.let { url -> { SupportIconBadge(icon = url, size = size) } },
    ) {
        MainIcon(icon = icon, placeholder = placeholder, size = size)
    }
}

@Composable
fun IconWithBadge(
    icon: Any?,
    placeholder: String? = null,
    size: Dp = listItemIconSize,
    badge: @Composable () -> Unit,
) {
    icon ?: return
    IconWithBadge(size = size, badge = badge) {
        MainIcon(icon = icon, placeholder = placeholder, size = size)
    }
}

@Composable
fun IconWithBadge(
    size: Dp = listItemIconSize,
    badge: (@Composable () -> Unit)? = null,
    content: @Composable () -> Unit,
) {
    BadgedBox(size = size, badge = badge, content = content)
}

private const val BADGE_CONTENT_SIZE_RATIO = 2.6f
private const val BADGE_RING_WIDTH_RATIO = 32f
private const val BADGE_OFFSET_RATIO = 5f

private fun badgeContentSize(size: Dp): Dp = size / BADGE_CONTENT_SIZE_RATIO

@Composable
private fun MainIcon(
    icon: Any,
    placeholder: String?,
    size: Dp,
) {
    AsyncImage(
        model = icon,
        placeholderText = placeholder,
        contentDescription = "list_item_icon",
        size = size,
    )
}

@Composable
private fun SupportIconBadge(
    icon: Any,
    size: Dp,
) {
    AsyncImage(
        model = icon,
        size = badgeContentSize(size),
        contentDescription = null,
    )
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
            val badgeContentSize = badgeContentSize(size)
            val badgeRingWidth = size / BADGE_RING_WIDTH_RATIO
            val badgeSize = badgeContentSize + badgeRingWidth * 2
            val badgeOffset = badgeSize / BADGE_OFFSET_RATIO
            Box(
                modifier = Modifier
                    .offset(badgeOffset, badgeOffset)
                    .size(badgeSize)
                    .align(Alignment.BottomEnd)
                    .background(MaterialTheme.colorScheme.background, CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                badge()
            }
        }
    }
}
