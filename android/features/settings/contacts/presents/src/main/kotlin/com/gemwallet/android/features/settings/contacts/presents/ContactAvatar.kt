package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAvatarState
import com.gemwallet.android.ui.components.image.AsyncImage
import com.gemwallet.android.ui.components.image.AvatarScale
import com.gemwallet.android.ui.components.image.EmojiView
import com.gemwallet.android.ui.components.image.InitialsAvatar
import com.gemwallet.android.ui.components.image.RemoveBadge
import com.gemwallet.android.ui.components.image.walletImageModel
import com.gemwallet.android.ui.icons.AppIcons
import com.gemwallet.android.ui.theme.listItemIconSize

@Composable
internal fun ContactAvatar(
    name: String,
    avatar: ContactAvatarState,
    modifier: Modifier = Modifier,
    size: Dp = listItemIconSize,
    onRemove: (() -> Unit)? = null,
) {
    val initials = name.trim().take(2).uppercase()
    Box(modifier = modifier) {
        when (avatar) {
            ContactAvatarState.Empty -> InitialsAvatar(
                text = initials,
                size = size,
                placeholder = AppIcons.Person,
            )
            is ContactAvatarState.Image -> AsyncImage(
                model = walletImageModel(LocalContext.current, avatar.imageUrl),
                size = size,
                placeholderText = initials,
            )
            is ContactAvatarState.Emoji -> EmojiView(
                emoji = avatar.emoji,
                modifier = Modifier.size(size),
                background = Color(avatar.backgroundColor),
                scale = AvatarScale.EMOJI,
            )
        }
        if (onRemove != null) {
            RemoveBadge(onClick = onRemove)
        }
    }
}
