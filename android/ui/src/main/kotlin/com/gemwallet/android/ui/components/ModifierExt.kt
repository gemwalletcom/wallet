package com.gemwallet.android.ui.components

import androidx.compose.foundation.clickable as foundationClickable
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Shape

@Composable
fun Modifier.clickable(onClick: () -> Unit): Modifier {
    return clickable(enabled = true, onClick = onClick)
}

@Composable
fun Modifier.clickable(
    enabled: Boolean,
    onClick: () -> Unit,
    shape: Shape? = null,
): Modifier {
    val resolvedShape = shape ?: MaterialTheme.shapes.medium
    return Modifier
        .clip(resolvedShape)
        .then(this)
        .foundationClickable(enabled = enabled, onClick = onClick)
}
