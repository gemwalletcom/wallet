package com.gemwallet.android.ui.components.list_head

import androidx.compose.foundation.LocalIndication
import androidx.compose.foundation.clickable
import androidx.compose.foundation.indication
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape

fun Modifier.headerClick(interactionSource: MutableInteractionSource, onClick: (() -> Unit)?): Modifier = if (onClick == null) this else clickable(interactionSource = interactionSource, indication = null, onClick = onClick)

@Composable
fun BoxScope.HeaderHighlight(interactionSource: MutableInteractionSource, shape: Shape) {
    Box(
        modifier = Modifier
            .matchParentSize()
            .clip(shape)
            .indication(interactionSource, LocalIndication.current),
    )
}
