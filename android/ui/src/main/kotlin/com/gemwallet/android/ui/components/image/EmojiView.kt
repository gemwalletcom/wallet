package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity

const val EMOJI_AVATAR_SCALE = 0.7f

@Composable
fun EmojiView(
    emoji: String,
    modifier: Modifier = Modifier,
    background: Color,
    scale: Float,
) {
    BoxWithConstraints(
        modifier = modifier
            .clip(CircleShape)
            .background(background),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = emoji,
            fontSize = with(LocalDensity.current) { (maxWidth * scale).toSp() },
        )
    }
}
