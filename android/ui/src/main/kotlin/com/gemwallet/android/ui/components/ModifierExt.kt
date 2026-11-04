package com.gemwallet.android.ui.components

import android.os.Build
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.isSensitiveData
import androidx.compose.ui.semantics.semantics
import androidx.compose.foundation.clickable as foundationClickable

@Composable
fun Modifier.clickable(onClick: () -> Unit): Modifier = clickable(enabled = true, onClick = onClick)

@Composable
fun Modifier.clickable(enabled: Boolean, onClick: () -> Unit, shape: Shape? = null): Modifier {
    val resolvedShape = shape ?: MaterialTheme.shapes.medium
    return Modifier
        .clip(resolvedShape)
        .then(this)
        .foundationClickable(enabled = enabled, onClick = onClick)
}

fun Modifier.consumeAllPointerEvents(): Modifier = pointerInput(Unit) {
    awaitPointerEventScope {
        while (true) {
            awaitPointerEvent().changes.forEach { it.consume() }
        }
    }
}

@Composable
fun Modifier.keyboardFollowsFocus(): Modifier {
    val keyboardController = LocalSoftwareKeyboardController.current
    return onFocusChanged { if (it.hasFocus) keyboardController?.show() else keyboardController?.hide() }
}

fun Modifier.sensitiveData(): Modifier =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
        semantics { isSensitiveData = true }
    } else {
        clearAndSetSemantics { }
    }
