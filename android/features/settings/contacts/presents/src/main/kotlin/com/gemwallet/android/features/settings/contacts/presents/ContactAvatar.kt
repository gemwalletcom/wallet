package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ContactAvatarState
import com.gemwallet.android.features.settings.contacts.viewmodels.models.image
import com.gemwallet.android.ui.components.image.ListItemImageView
import com.gemwallet.android.ui.components.image.RemoveBadge
import com.gemwallet.android.ui.theme.listItemIconSize

@Composable
internal fun ContactAvatar(initials: String, avatar: ContactAvatarState, modifier: Modifier = Modifier, size: Dp = listItemIconSize, onRemove: (() -> Unit)? = null) {
    Box(modifier = modifier) {
        ListItemImageView(image = avatar.image(initials), size = size)
        if (onRemove != null) {
            RemoveBadge(onClick = onRemove)
        }
    }
}
