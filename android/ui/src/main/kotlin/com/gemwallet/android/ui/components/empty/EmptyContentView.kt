package com.gemwallet.android.ui.components.empty

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.vectorResource

@Composable
fun EmptyContentView(
    type: EmptyContentType,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val model = remember(type) { type.uiModel(context) }
    val (icon, iconVector) = when (val image = model.image) {
        is EmptyStateImage.Drawable -> painterResource(image.id) to null
        is EmptyStateImage.Vector -> null to ImageVector.vectorResource(image.id)
    }
    EmptyStateView(
        title = model.title,
        description = model.description,
        icon = icon,
        iconVector = iconVector,
        buttons = model.buttons,
        modifier = modifier,
    )
}
