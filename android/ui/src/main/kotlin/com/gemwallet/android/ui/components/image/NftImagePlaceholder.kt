package com.gemwallet.android.ui.components.image

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.gemwallet.android.ui.components.progress.CircularProgressIndicator20
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.paddingLarge

@Composable
fun NftImagePlaceholder(
    modifier: Modifier = Modifier,
    name: String = "",
) {
    BoxWithConstraints(
        modifier = modifier.background(MaterialTheme.colorScheme.surfaceContainerHighest),
        contentAlignment = Alignment.Center,
    ) {
        val minSide = minOf(maxWidth, maxHeight)
        val showName = minSide > Metrics.nameVisibilityThreshold
        val circleRatio = if (showName) Metrics.CIRCLE_SIZE_RATIO_WITH_TEXT else Metrics.CIRCLE_SIZE_RATIO_DEFAULT
        val circleSize = minSide * circleRatio
        val iconSize = circleSize * Metrics.ICON_SIZE_RATIO

        Column(
            verticalArrangement = Arrangement.spacedBy(paddingDefault),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Box(
                modifier = Modifier
                    .size(circleSize)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.outlineVariant),
                contentAlignment = Alignment.Center,
            ) {
                Icon(
                    imageVector = Icons.Outlined.Image,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.outline,
                    modifier = Modifier.size(iconSize),
                )
            }
            if (showName && name.isNotBlank()) {
                Text(
                    text = name,
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    textAlign = TextAlign.Center,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.padding(horizontal = paddingLarge),
                )
            }
        }
    }
}

@Composable
internal fun NftImageLoading(modifier: Modifier = Modifier) {
    Box(
        modifier = modifier.background(MaterialTheme.colorScheme.surfaceContainerHighest),
        contentAlignment = Alignment.Center,
    ) {
        CircularProgressIndicator20(color = MaterialTheme.colorScheme.outline)
    }
}

private object Metrics {
    val nameVisibilityThreshold = 250.dp
    const val CIRCLE_SIZE_RATIO_WITH_TEXT = 0.30f
    const val CIRCLE_SIZE_RATIO_DEFAULT = 0.35f
    const val ICON_SIZE_RATIO = 0.45f
}
