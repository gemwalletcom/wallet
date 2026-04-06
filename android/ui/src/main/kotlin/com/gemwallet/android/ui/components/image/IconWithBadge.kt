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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
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
    badgeBackgroundColor: Color? = null,
) {
    IconWithBadge(
        icon = asset.getIconUrl(),
        placeholder = asset.type.string,
        supportIcon = asset.getSupportIconUrl(),
        size = size,
        badgeBackgroundColor = badgeBackgroundColor,
    )
}

@Composable
fun IconWithBadge(
    icon: Any?,
    placeholder: String? = null,
    supportIcon: Any? = null,
    size: Dp = listItemIconSize,
    badgeBackgroundColor: Color? = null,
) {
    icon ?: return
    IconWithBadge(
        size = size,
        badgeBackgroundColor = badgeBackgroundColor,
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
    badgeBackgroundColor: Color? = null,
    badge: @Composable () -> Unit,
) {
    icon ?: return
    IconWithBadge(size = size, badgeBackgroundColor = badgeBackgroundColor, badge = badge) {
        MainIcon(icon = icon, placeholder = placeholder, size = size)
    }
}

@Composable
fun IconWithBadge(
    size: Dp = listItemIconSize,
    badgeBackgroundColor: Color? = null,
    badge: (@Composable () -> Unit)? = null,
    content: @Composable () -> Unit,
) {
    BadgedBox(
        size = size,
        badgeBackgroundColor = badgeBackgroundColor ?: MaterialTheme.colorScheme.background,
        badge = badge,
        content = content,
    )
}

private const val BADGE_CONTENT_SIZE_RATIO = 2.6f
private const val LARGE_BADGE_CONTENT_SIZE_RATIO = 3f
private const val BADGE_RING_WIDTH_RATIO = 32f
private const val BADGE_OFFSET_RATIO = 5f
private val LARGE_BADGE_THRESHOLD = 48.dp
private val MAX_BADGE_RING_WIDTH = 2.dp

internal data class BadgeLayout(
    val contentSize: Dp,
    val ringWidth: Dp,
    val badgeSize: Dp,
    val offset: Dp,
)

internal fun badgeLayout(size: Dp): BadgeLayout {
    val contentSize = if (size <= LARGE_BADGE_THRESHOLD) {
        size / BADGE_CONTENT_SIZE_RATIO
    } else {
        size / LARGE_BADGE_CONTENT_SIZE_RATIO
    }
    val ringWidth = (size / BADGE_RING_WIDTH_RATIO).coerceAtMost(MAX_BADGE_RING_WIDTH)
    val badgeSize = contentSize + ringWidth * 2
    val offset = badgeSize / BADGE_OFFSET_RATIO
    return BadgeLayout(
        contentSize = contentSize,
        ringWidth = ringWidth,
        badgeSize = badgeSize,
        offset = offset,
    )
}

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
        size = badgeLayout(size).contentSize,
        contentDescription = null,
    )
}

@Composable
internal fun BadgeCircle(
    size: Dp,
    color: Color,
    content: @Composable () -> Unit,
) {
    Box(
        modifier = Modifier
            .size(badgeLayout(size).contentSize)
            .clip(CircleShape)
            .background(color),
        contentAlignment = Alignment.Center,
    ) {
        content()
    }
}

@Composable
private fun BadgedBox(
    size: Dp,
    badgeBackgroundColor: Color,
    badge: (@Composable () -> Unit)?,
    content: @Composable () -> Unit,
) {
    Box {
        content()
        if (badge != null) {
            val layout = badgeLayout(size)
            Box(
                modifier = Modifier
                    .offset(layout.offset, layout.offset)
                    .size(layout.badgeSize)
                    .align(Alignment.BottomEnd)
                    .background(badgeBackgroundColor, CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                badge()
            }
        }
    }
}
