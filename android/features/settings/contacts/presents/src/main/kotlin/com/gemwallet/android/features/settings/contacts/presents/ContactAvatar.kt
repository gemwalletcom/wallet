package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.components.EmojiView
import com.gemwallet.android.ui.theme.listItemIconSize
import com.gemwallet.android.ui.theme.secondaryFaded

private const val EMOJI_SCALE = 0.55f

@Composable
internal fun ContactAvatar(
    name: String,
    emoji: String?,
    modifier: Modifier = Modifier,
    size: Dp = listItemIconSize,
) {
    if (emoji.isNullOrBlank()) {
        Box(
            modifier = modifier
                .size(size)
                .clip(CircleShape)
                .background(MaterialTheme.colorScheme.secondaryFaded),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = name.take(2).uppercase(),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
    } else {
        EmojiView(
            emoji = emoji,
            modifier = modifier.size(size),
            background = MaterialTheme.colorScheme.secondaryFaded,
            scale = EMOJI_SCALE,
        )
    }
}
