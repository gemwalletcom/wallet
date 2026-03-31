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
import com.gemwallet.android.ui.theme.space2
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

private const val BADGE_SIZE_RATIO = 2.5f

@Composable
private fun BadgedIcon(
    icon: Any,
    placeholder: String?,
    size: Dp,
    badge: (@Composable () -> Unit)? = null,
) {
    Box {
        AsyncImage(
            model = icon,
            placeholderText = placeholder,
            contentDescription = "list_item_icon",
            size = size,
        )
        if (badge != null) {
            Box(
                modifier = Modifier
                    .offset(space2, space2)
                    .size(size / BADGE_SIZE_RATIO)
                    .align(Alignment.BottomEnd)
                    .border(1.5.dp, MaterialTheme.colorScheme.surface, CircleShape),
            ) {
                badge()
            }
        }
    }
}
