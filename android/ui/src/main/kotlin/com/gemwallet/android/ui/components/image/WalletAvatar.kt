package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.paddingSmall
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.tinyIconSize

private val removeBadgeSize = 28.dp

@Composable
fun WalletAvatar(
    imageUrl: String?,
    placeholder: Any?,
    size: Dp,
    modifier: Modifier = Modifier,
    supportIcon: Any? = null,
    onClick: (() -> Unit)? = null,
    onRemove: (() -> Unit)? = null,
) {
    val clickModifier = if (onClick != null) {
        Modifier.clickable(
            interactionSource = remember { MutableInteractionSource() },
            indication = ripple(bounded = false, radius = size / 2),
            onClick = onClick,
        )
    } else {
        Modifier
    }
    Box(modifier = modifier.then(clickModifier)) {
        IconWithBadge(
            icon = walletImageModel(LocalContext.current, imageUrl) ?: placeholder,
            supportIcon = supportIcon,
            size = size,
        )
        if (onRemove != null) {
            RemoveBadge(onClick = onRemove)
        }
    }
}

@Composable
private fun BoxScope.RemoveBadge(onClick: () -> Unit) {
    IconButton(
        onClick = onClick,
        modifier = Modifier
            .align(Alignment.TopEnd)
            .offset(x = paddingSmall, y = -paddingSmall),
    ) {
        Box(
            modifier = Modifier
                .size(removeBadgeSize)
                .shadow(elevation = space2, shape = CircleShape)
                .background(MaterialTheme.colorScheme.surface, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                imageVector = AppIcons.Close,
                contentDescription = null,
                modifier = Modifier.size(tinyIconSize),
            )
        }
    }
}
