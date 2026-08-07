package com.gemwallet.android.ui.components.buttons

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import com.gemwallet.android.ui.theme.space2
import com.gemwallet.android.ui.theme.space8

@Composable
fun IndicatorButton(
    imageVector: ImageVector,
    showsIndicator: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    IconButton(onClick = onClick, modifier = modifier) {
        Box {
            Icon(
                imageVector = imageVector,
                tint = MaterialTheme.colorScheme.onSurface,
                contentDescription = null,
            )
            if (showsIndicator) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .background(MaterialTheme.colorScheme.surface, CircleShape)
                        .padding(space2)
                        .size(space8)
                        .background(MaterialTheme.colorScheme.primary, CircleShape)
                )
            }
        }
    }
}
