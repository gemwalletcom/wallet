package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.Dp

@Composable
fun WalletAvatar(
    imageUrl: String?,
    placeholder: Any?,
    size: Dp,
    modifier: Modifier = Modifier,
    supportIcon: Any? = null,
    onClick: (() -> Unit)? = null,
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
    }
}
