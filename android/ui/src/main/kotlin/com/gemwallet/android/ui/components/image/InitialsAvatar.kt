package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import com.gemwallet.android.ui.theme.secondaryFaded

@Composable
fun InitialsAvatar(
    text: String,
    size: Dp,
    modifier: Modifier = Modifier,
    placeholder: ImageVector? = null,
) {
    Box(
        modifier = modifier
            .size(size)
            .clip(CircleShape)
            .background(MaterialTheme.colorScheme.secondaryFaded),
        contentAlignment = Alignment.Center,
    ) {
        if (text.isEmpty() && placeholder != null) {
            Icon(
                imageVector = placeholder,
                contentDescription = null,
                modifier = Modifier.size(size * AvatarScale.EMOJI),
                tint = MaterialTheme.colorScheme.outline,
            )
        } else {
            Text(
                text = text,
                style = MaterialTheme.typography.titleMedium,
                fontSize = with(LocalDensity.current) { (size * AvatarScale.INITIALS).toSp() },
                color = MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}
